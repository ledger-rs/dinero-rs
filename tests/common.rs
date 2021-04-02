use dinero::run_application;

pub fn test_args(args: &[&str]) {
    let mut function_args: Vec<&str> = vec!["testing"];
    for arg in args {
        function_args.push(arg);
    }
    let res = run_application(function_args.iter().map(|x| x.to_string()).collect());
    assert!(res.is_ok());
}

pub fn test_err(args: &[&str]) {
    let mut function_args: Vec<&str> = vec!["testing"];
    for arg in args {
        function_args.push(arg);
    }
    let res = run_application(function_args.iter().map(|x| x.to_string()).collect());
    assert!(res.is_err());
}
