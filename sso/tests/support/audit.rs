#[macro_export]
macro_rules! audit_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_audit_list_created_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let client = client_create(Some(&service_key.value));
            let path = "test_1";
            let limit = 3;

            let a1 = client
                .audit_create(AuditCreateRequest::new(path, Value::Null, None, None))
                .unwrap()
                .data;
            client
                .audit_create(AuditCreateRequest::new(path, Value::Null, None, None))
                .unwrap();
            client
                .audit_create(AuditCreateRequest::new(path, Value::Null, None, None))
                .unwrap();
            client
                .audit_create(AuditCreateRequest::new(path, Value::Null, None, None))
                .unwrap();
            client
                .audit_create(AuditCreateRequest::new(path, Value::Null, None, None))
                .unwrap();

            let res1 = client
                .audit_list(AuditListRequest {
                    ge: Some(a1.created_at),
                    le: None,
                    offset_id: None,
                    limit: Some(limit),
                    type_: None,
                    service_id: None,
                    user_id: None,
                })
                .unwrap();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = res1.data[0].id;
            let r1_2 = res1.data[1].id;
            let r1_3 = res1.data[2].id;
            assert_eq!(r1_1, a1.id);
            let a1 = client.audit_read(r1_1).unwrap().data;

            let res2 = client
                .audit_list(AuditListRequest {
                    ge: Some(a1.created_at),
                    le: None,
                    offset_id: Some(a1.id),
                    limit: Some(limit),
                    type_: None,
                    service_id: None,
                    user_id: None,
                })
                .unwrap();
            assert_eq!(res2.data.len(), 3);
            let r2_2 = res2.data[0].id;
            let r2_3 = res2.data[1].id;
            let r2_4 = res2.data[2].id;
            assert_eq!(r2_2, r1_2);
            assert_eq!(r2_3, r1_3);
            let a2 = client.audit_read(r2_2).unwrap().data;

            let res3 = client
                .audit_list(AuditListRequest {
                    ge: Some(a2.created_at),
                    le: None,
                    offset_id: Some(a2.id),
                    limit: Some(limit),
                    type_: None,
                    service_id: None,
                    user_id: None,
                })
                .unwrap();
            assert_eq!(res3.data.len(), 3);
            let r3_3 = res3.data[0].id;
            let r3_4 = res3.data[1].id;
            let r3_5 = res3.data[2].id;
            assert_eq!(r3_3, r2_3);
            assert_eq!(r3_4, r2_4);
            let a5 = client.audit_read(r3_5).unwrap().data;

            let res4 = client
                .audit_list(AuditListRequest {
                    ge: None,
                    le: Some(a5.created_at),
                    offset_id: Some(a5.id),
                    limit: Some(limit),
                    type_: None,
                    service_id: None,
                    user_id: None,
                })
                .unwrap();
            assert_eq!(res4.data.len(), 3);
            let r4_2 = res4.data[0].id;
            let r4_3 = res4.data[1].id;
            let r4_4 = res4.data[2].id;
            assert_eq!(r4_2, r2_2);
            assert_eq!(r4_3, r3_3);
            assert_eq!(r4_4, r3_4);
            let a4 = client.audit_read(r4_4).unwrap().data;

            let res5 = client
                .audit_list(AuditListRequest {
                    ge: None,
                    le: Some(a4.created_at),
                    offset_id: Some(a4.id),
                    limit: Some(limit),
                    type_: None,
                    service_id: None,
                    user_id: None,
                })
                .unwrap();
            assert_eq!(res5.data.len(), 3);
            let r5_1 = res5.data[0].id;
            let r5_2 = res5.data[1].id;
            let r5_3 = res5.data[2].id;
            assert_eq!(r5_1, r1_1);
            assert_eq!(r5_2, r4_2);
            assert_eq!(r5_3, r4_3);
        }

        #[test]
        #[ignore]
        fn api_audit_list_created_and_ok() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let client = client_create(Some(&service_key.value));
            let path = "test_1";
            let limit = 3;

            let a1 = client
                .audit_create(AuditCreateRequest::new(path, Value::Null, None, None))
                .unwrap()
                .data;
            client
                .audit_create(AuditCreateRequest::new(path, Value::Null, None, None))
                .unwrap();
            client
                .audit_create(AuditCreateRequest::new(path, Value::Null, None, None))
                .unwrap();

            let res1 = client
                .audit_list(AuditListRequest {
                    ge: Some(a1.created_at),
                    le: None,
                    offset_id: None,
                    limit: Some(limit),
                    type_: None,
                    service_id: None,
                    user_id: None,
                })
                .unwrap();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = res1.data[0].id;
            let r1_2 = res1.data[1].id;
            let r1_3 = res1.data[2].id;
            assert_eq!(r1_1, a1.id);
            let a1 = client.audit_read(r1_1).unwrap().data;
            let a3 = client.audit_read(r1_3).unwrap().data;

            let res2 = client
                .audit_list(AuditListRequest {
                    ge: Some(a1.created_at),
                    le: Some(a3.created_at),
                    offset_id: Some(a1.id),
                    limit: Some(limit),
                    type_: None,
                    service_id: None,
                    user_id: None,
                })
                .unwrap();
            assert_eq!(res2.data.len(), 2);
            let r2_2 = res2.data[0].id;
            let r2_3 = res2.data[1].id;
            assert_eq!(r2_2, r1_2);
            assert_eq!(r2_3, r1_3);
        }

        #[test]
        #[ignore]
        fn api_audit_read_not_found_does_not_exist() {
            let client = client_create(None);
            let (_service, service_key) = service_key_create(&client);
            let client = client_create(Some(&service_key.value));

            let res = client.audit_read(Uuid::nil()).unwrap_err();
            assert_eq!(res, ClientError::NotFound);
        }

        #[test]
        #[ignore]
        fn api_audit_read_not_found_masked_by_service() {
            let client = client_create(None);
            let (_service1, service_key1) = service_key_create(&client);
            let (_service2, service_key2) = service_key_create(&client);

            let client = client_create(Some(&service_key1.value));
            let a1 = client
                .audit_create(AuditCreateRequest::new("test_1", Value::Null, None, None))
                .unwrap()
                .data;

            let client = client_create(Some(&service_key2.value));
            let res = client.audit_read(a1.id).unwrap_err();
            assert_eq!(res, ClientError::NotFound);
        }
    };
}