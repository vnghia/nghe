mod tests {
    use nghe_proc_macros::{add_axum_response, add_subsonic_response};
    use nghe_types::constant;
    use serde_json::{json, to_value};

    #[test]
    fn test_ser_success_empty() {
        #[add_subsonic_response]
        struct TestBody {}
        add_axum_response!(TestBody);

        assert_eq!(
            to_value(Into::<TestSubsonicResponse>::into(TestBody {})).unwrap(),
            json!({
                "subsonic-response": {
                    "status": "ok",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_NAME,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        )
    }

    #[test]
    fn test_ser_success() {
        #[add_subsonic_response]
        struct TestBody {
            a: u16,
        }
        add_axum_response!(TestBody);
        let a = 10;

        assert_eq!(
            to_value(Into::<TestSubsonicResponse>::into(TestBody { a })).unwrap(),
            json!({
                "subsonic-response": {
                    "a": a,
                    "status": "ok",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_NAME,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        )
    }

    #[test]
    fn test_ser_success_camel_case() {
        #[add_subsonic_response]
        struct TestBody {
            camel_case: u16,
        }
        add_axum_response!(TestBody);
        let camel_case = 10;

        assert_eq!(
            to_value(Into::<TestSubsonicResponse>::into(TestBody { camel_case })).unwrap(),
            json!({
                "subsonic-response": {
                    "camelCase": camel_case,
                    "status": "ok",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_NAME,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        )
    }

    #[test]
    fn test_ser_error_empty() {
        #[add_subsonic_response(success = false)]
        struct TestBody {}
        add_axum_response!(TestBody);

        assert_eq!(
            to_value(Into::<TestSubsonicResponse>::into(TestBody {})).unwrap(),
            json!({
                "subsonic-response": {
                    "status": "failed",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_NAME,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        )
    }

    #[test]
    fn test_ser_error() {
        #[add_subsonic_response(success = false)]
        struct TestBody {
            a: u16,
        }
        add_axum_response!(TestBody);
        let a = 10;

        assert_eq!(
            to_value(Into::<TestSubsonicResponse>::into(TestBody { a })).unwrap(),
            json!({
                "subsonic-response": {
                    "a": a,
                    "status": "failed",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_NAME,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        )
    }
}
