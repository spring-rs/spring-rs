use log::debug;
use spring_opendal::config::*;
use spring_opendal::OpenDALPlugin;

#[test_log::test]
fn blocking() {
    let config = OpenDALConfig {
        scheme: "memory".to_string(),
        options: None,
        layers: Some(vec![Layers::Blocking]),
    };

    debug!("config: {:?}", config);

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.spawn_blocking(move || {
        let op = OpenDALPlugin::operator(config).unwrap().blocking();
        op.write("test", b"test".to_vec()).unwrap();
        let res = op.read("test").unwrap();

        assert_eq!(res.to_vec(), b"test".to_vec());
    });
}
