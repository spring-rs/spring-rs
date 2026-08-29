use crate::config::CircuitBreakerConfig;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::Instant;

/// The externally visible state of a circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
struct CircuitBreakerPolicy {
    failure_rate_threshold: f64,
    sliding_window_size: usize,
    minimum_number_of_calls: usize,
    wait_duration_in_open_state: Duration,
    permitted_calls_in_half_open_state: u32,
}

impl TryFrom<CircuitBreakerConfig> for CircuitBreakerPolicy {
    type Error = CircuitBreakerConfigError;

    fn try_from(config: CircuitBreakerConfig) -> Result<Self, Self::Error> {
        if !(config.failure_rate_threshold.is_finite()
            && 0.0 < config.failure_rate_threshold
            && config.failure_rate_threshold <= 100.0)
        {
            return Err(CircuitBreakerConfigError::InvalidFailureRateThreshold(
                config.failure_rate_threshold,
            ));
        }
        if config.sliding_window_size == 0 {
            return Err(CircuitBreakerConfigError::ZeroSlidingWindowSize);
        }
        if config.minimum_number_of_calls == 0
            || config.minimum_number_of_calls > config.sliding_window_size
        {
            return Err(CircuitBreakerConfigError::InvalidMinimumNumberOfCalls);
        }
        if config.permitted_calls_in_half_open_state == 0 {
            return Err(CircuitBreakerConfigError::ZeroPermittedHalfOpenCalls);
        }

        Ok(Self {
            failure_rate_threshold: config.failure_rate_threshold,
            sliding_window_size: config.sliding_window_size as usize,
            minimum_number_of_calls: config.minimum_number_of_calls as usize,
            wait_duration_in_open_state: Duration::from_millis(config.wait_duration_in_open_state),
            permitted_calls_in_half_open_state: config.permitted_calls_in_half_open_state,
        })
    }
}

/// Invalid circuit breaker policy configuration.
#[derive(Debug, Error, PartialEq)]
pub enum CircuitBreakerConfigError {
    #[error("failure_rate_threshold must be finite and greater than 0 and at most 100, got {0}")]
    InvalidFailureRateThreshold(f64),
    #[error("sliding_window_size must be at least 1")]
    ZeroSlidingWindowSize,
    #[error("minimum_number_of_calls must be between 1 and sliding_window_size")]
    InvalidMinimumNumberOfCalls,
    #[error("permitted_calls_in_half_open_state must be at least 1")]
    ZeroPermittedHalfOpenCalls,
}

#[derive(Debug)]
enum State {
    Closed {
        outcomes: VecDeque<bool>,
    },
    Open {
        opened_at: Instant,
    },
    HalfOpen {
        admitted: u32,
        completed: u32,
        failures: u32,
    },
}

/// One configured circuit breaker, shared safely between asynchronous calls.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    name: Arc<str>,
    policy: Arc<CircuitBreakerPolicy>,
    state: Arc<Mutex<State>>,
}

impl CircuitBreaker {
    fn new(name: String, policy: CircuitBreakerPolicy) -> Self {
        Self {
            name: Arc::from(name),
            policy: Arc::new(policy),
            state: Arc::new(Mutex::new(State::Closed {
                outcomes: VecDeque::new(),
            })),
        }
    }

    /// Returns the current state, applying an elapsed open-state transition.
    pub async fn state(&self) -> CircuitBreakerState {
        let mut state = self.state.lock().await;
        self.transition_from_open_if_ready(&mut state);
        match *state {
            State::Closed { .. } => CircuitBreakerState::Closed,
            State::Open { .. } => CircuitBreakerState::Open,
            State::HalfOpen { .. } => CircuitBreakerState::HalfOpen,
        }
    }

    async fn acquire_permission(&self) -> Result<(), CallNotPermitted> {
        let mut state = self.state.lock().await;
        self.transition_from_open_if_ready(&mut state);
        match &mut *state {
            State::Closed { .. } => Ok(()),
            State::Open { .. } => Err(CallNotPermitted {
                name: self.name.to_string(),
            }),
            State::HalfOpen { admitted, .. }
                if *admitted < self.policy.permitted_calls_in_half_open_state =>
            {
                *admitted += 1;
                Ok(())
            }
            State::HalfOpen { .. } => Err(CallNotPermitted {
                name: self.name.to_string(),
            }),
        }
    }

    fn transition_from_open_if_ready(&self, state: &mut State) {
        let ready = matches!(state, State::Open { opened_at } if opened_at.elapsed() >= self.policy.wait_duration_in_open_state);
        if ready {
            *state = State::HalfOpen {
                admitted: 0,
                completed: 0,
                failures: 0,
            };
            tracing::debug!(circuit_breaker.name = %self.name, "circuit breaker entered half-open state");
        }
    }

    async fn record(&self, failed: bool) {
        let mut state = self.state.lock().await;
        match &mut *state {
            State::Closed { outcomes } => {
                outcomes.push_back(failed);
                if outcomes.len() > self.policy.sliding_window_size {
                    outcomes.pop_front();
                }
                if outcomes.len() >= self.policy.minimum_number_of_calls
                    && failure_rate(outcomes.iter().copied()) >= self.policy.failure_rate_threshold
                {
                    self.open(&mut state);
                }
            }
            State::HalfOpen {
                admitted: _,
                completed,
                failures,
            } => {
                *completed += 1;
                *failures += u32::from(failed);
                if *completed == self.policy.permitted_calls_in_half_open_state {
                    let rate = (*failures as f64 / *completed as f64) * 100.0;
                    if rate >= self.policy.failure_rate_threshold {
                        self.open(&mut state);
                    } else {
                        *state = State::Closed {
                            outcomes: VecDeque::new(),
                        };
                        tracing::debug!(circuit_breaker.name = %self.name, "circuit breaker closed");
                    }
                }
            }
            State::Open { .. } => {}
        }
    }

    fn open(&self, state: &mut State) {
        *state = State::Open {
            opened_at: Instant::now(),
        };
        tracing::debug!(circuit_breaker.name = %self.name, "circuit breaker opened");
    }
}

fn failure_rate(outcomes: impl Iterator<Item = bool>) -> f64 {
    let (failures, total) = outcomes.fold((0usize, 0usize), |(failures, total), failed| {
        (failures + usize::from(failed), total + 1)
    });
    (failures as f64 / total as f64) * 100.0
}

/// Error returned when an open circuit rejects a call.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("circuit breaker `{name}` does not permit calls")]
pub struct CallNotPermitted {
    name: String,
}

impl CallNotPermitted {
    /// Name of the circuit breaker that rejected the call.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Result error from the programmatic circuit breaker API.
#[derive(Debug, Error)]
pub enum CircuitBreakerError<E> {
    #[error(transparent)]
    CallNotPermitted(#[from] CallNotPermitted),
    #[error("guarded operation failed")]
    Operation(E),
}

/// Executes one asynchronous operation through a circuit breaker.
pub async fn execute<F, Fut, T, E, P>(
    circuit_breaker: CircuitBreaker,
    operation: F,
    record_failure: P,
) -> Result<T, CircuitBreakerError<E>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: Fn(&E) -> bool,
{
    circuit_breaker.acquire_permission().await?;
    match operation().await {
        Ok(value) => {
            circuit_breaker.record(false).await;
            Ok(value)
        }
        Err(error) => {
            circuit_breaker.record(record_failure(&error)).await;
            Err(CircuitBreakerError::Operation(error))
        }
    }
}

/// Registry of named circuit breakers.
#[derive(Debug, Clone, Default)]
pub struct CircuitBreakerRegistry {
    circuit_breakers: HashMap<String, CircuitBreaker>,
}

impl CircuitBreakerRegistry {
    pub(crate) fn from_configs(
        configs: HashMap<String, CircuitBreakerConfig>,
    ) -> Result<Self, NamedCircuitBreakerConfigError> {
        let circuit_breakers = configs
            .into_iter()
            .map(
                |(name, config)| match CircuitBreakerPolicy::try_from(config) {
                    Ok(policy) => Ok((name.clone(), CircuitBreaker::new(name, policy))),
                    Err(source) => Err(NamedCircuitBreakerConfigError { name, source }),
                },
            )
            .collect::<Result<_, _>>()?;
        Ok(Self { circuit_breakers })
    }

    /// Gets a named circuit breaker.
    pub fn get(&self, name: &str) -> Option<CircuitBreaker> {
        self.circuit_breakers.get(name).cloned()
    }
}

#[derive(Debug, Error)]
#[error("invalid circuit breaker policy `{name}`: {source}")]
pub(crate) struct NamedCircuitBreakerConfigError {
    name: String,
    #[source]
    source: CircuitBreakerConfigError,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn breaker(config: CircuitBreakerConfig) -> CircuitBreaker {
        CircuitBreaker::new(
            "test".into(),
            CircuitBreakerPolicy::try_from(config).unwrap(),
        )
    }

    #[tokio::test]
    async fn opens_after_the_configured_failure_rate_is_reached() {
        let breaker = breaker(CircuitBreakerConfig {
            sliding_window_size: 2,
            minimum_number_of_calls: 2,
            ..CircuitBreakerConfig::default()
        });

        for _ in 0..2 {
            let result = execute(
                breaker.clone(),
                || async { Err::<(), _>("failed") },
                |_| true,
            )
            .await;
            assert!(matches!(
                result,
                Err(CircuitBreakerError::Operation("failed"))
            ));
        }

        assert_eq!(breaker.state().await, CircuitBreakerState::Open);
        let result = execute(breaker, || async { Ok::<_, &str>(()) }, |_| true).await;
        assert!(matches!(
            result,
            Err(CircuitBreakerError::CallNotPermitted(_))
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn successful_half_open_probe_closes_the_circuit() {
        let breaker = breaker(CircuitBreakerConfig {
            sliding_window_size: 1,
            minimum_number_of_calls: 1,
            wait_duration_in_open_state: 100,
            permitted_calls_in_half_open_state: 1,
            ..CircuitBreakerConfig::default()
        });
        let _ = execute(
            breaker.clone(),
            || async { Err::<(), _>("failed") },
            |_| true,
        )
        .await;
        tokio::time::advance(Duration::from_millis(100)).await;

        let result = execute(breaker.clone(), || async { Ok::<_, &str>(42) }, |_| true).await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(breaker.state().await, CircuitBreakerState::Closed);
    }

    #[test]
    fn rejects_an_invalid_minimum_call_count() {
        let result = CircuitBreakerPolicy::try_from(CircuitBreakerConfig {
            sliding_window_size: 2,
            minimum_number_of_calls: 3,
            ..CircuitBreakerConfig::default()
        });
        assert_eq!(
            result.unwrap_err(),
            CircuitBreakerConfigError::InvalidMinimumNumberOfCalls
        );
    }
}
