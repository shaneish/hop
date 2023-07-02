use bhop::args::Request;
use std::env;
use serial_test::serial;

fn setup_args(args: &[&str]) {
    let args = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    env::set_var("BHOP_TEST_ARGS", args.join(" "));
}

#[test]
#[serial]
fn test_request_parse_add() {
    setup_args(&["hp", "add", "shortcut", "location"]);
    let request = Request::parse();
    assert_eq!(
        request,
        Request::Add("shortcut".to_string(), Some("location".to_string()))
    );
}

#[test]
#[serial]
fn test_request_parse_remove() {
    setup_args(&["hp", "rm", "shortcut"]);
    let request = Request::parse();
    assert_eq!(request, Request::Remove("shortcut".to_string()));
}

#[test]
#[serial]
fn test_request_parse_help() {
    setup_args(&["hp", "help"]);
    let request = Request::parse();
    assert_eq!(request, Request::Passthrough("__bhop_help__".to_string()));
}

