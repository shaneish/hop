use bhop::sanitize;

#[test]
fn sanitize_correctly_replaces_backslashes() {
    let path = if cfg!(windows) {
        r"C:\Path\To\File"
    } else {
        r"/Path/To/File"
    };
    let result = sanitize(path).unwrap();
    assert_eq!(
        result,
        if cfg!(windows) {
            "C:/Path/To/File"
        } else {
            "/Path/To/File"
        }
    );
}
