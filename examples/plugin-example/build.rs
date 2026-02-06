pub fn main() {
    let _ = spring_apalis::ApalisPlugin;
    let _ = spring_grpc::GrpcPlugin;
    let _ = spring_job::JobPlugin;
    let _ = spring_mail::MailPlugin;
    let _ = spring_opendal::OpenDALPlugin;
    let _ = spring_opentelemetry::OpenTelemetryPlugin;
    let _ = spring_postgres::PgPlugin;
    let _ = spring_redis::RedisPlugin;
    let _ = spring_sea_orm::SeaOrmPlugin;
    let _ = spring_sqlx::SqlxPlugin;
    let _ = spring_sa_token::SaTokenPlugin;
    let _ = spring_stream::StreamPlugin;
    let _ = spring_web::WebPlugin;

    let r = spring::config::write_merged_schema_to_file("../../target/config-schema.json");
    println!("{r:?}")
}
