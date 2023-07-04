use crate::node::util::util::parse_classpath_args;

#[test]
fn parse_classpath_args_basic() {
    let cp = vec!["/lib/abc/*".to_string(), "/lib/def.jar".to_string()];
    let mut args = vec![];

    let classpath = parse_classpath_args(&cp, &mut args);

    if cfg!(windows) {
        assert_eq!(classpath, "-Djava.class.path=/lib/abc/*;/lib/def.jar");
    } else {
        assert_eq!(classpath, "-Djava.class.path=/lib/abc/*:/lib/def.jar");
    }
}

#[test]
fn parse_classpath_args_with_input_args() {
    let cp = vec!["/lib/abc/*".to_string(), "/lib/def.jar".to_string()];
    let mut args = vec![
        "-Xmx=2".to_string(),
        "-Djava.class.path=/var/abc;/var/def".to_string(),
        "-Xms=5".to_string(),
    ];

    let classpath = parse_classpath_args(&cp, &mut args);

    if cfg!(windows) {
        assert_eq!(
            classpath,
            "-Djava.class.path=/lib/abc/*;/lib/def.jar;/var/abc;/var/def"
        );
    } else {
        assert_eq!(
            classpath,
            "-Djava.class.path=/lib/abc/*:/lib/def.jar:/var/abc:/var/def"
        );
    }

    args = vec!["-Djava.class.path=/home/abc;/home/def".to_string()];
    let classpath = parse_classpath_args(&cp, &mut args);

    if cfg!(windows) {
        assert_eq!(
            classpath,
            "-Djava.class.path=/lib/abc/*;/lib/def.jar;/home/abc;/home/def"
        );
    } else {
        assert_eq!(
            classpath,
            "-Djava.class.path=/lib/abc/*:/lib/def.jar:/home/abc:/home/def"
        );
    }
}
