// Set darkmode
document.getElementById('mode').addEventListener('click', () => {
  document.body.classList.toggle('dark');
  var theme = document.body.classList.contains('dark') ? 'dark' : 'light';
  localStorage.setItem('theme', theme);
  updateUtterancesTheme('github-' + theme);
});

// enforce local storage setting but also fallback to user-agent preferences
if (localStorage.getItem('theme') === 'dark' || (!localStorage.getItem('theme') && window.matchMedia("(prefers-color-scheme: dark)").matches)) {
  document.body.classList.add('dark');
  updateUtterancesTheme('github-dark');
}

function updateUtterancesTheme(theme) {
  const iframe = document.querySelector('iframe.utterances-frame');
  if (iframe) {
    iframe.contentWindow.postMessage({
      type: 'set-theme',
      theme: theme
    }, 'https://utteranc.es');
  }
}