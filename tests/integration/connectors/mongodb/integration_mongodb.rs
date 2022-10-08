use actix_http::body::BoxBody;
use serial_test::serial;
use teo::core::graph::Graph;
use actix_web::{test, App, error::Error};
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use teo::app::app::ServerConfiguration;
use teo::app::serve::make_app;
use crate::helpers::is_object_id;


async fn make_mongodb_graph() -> Graph {
    Graph::new(|g| {
        g.data_source().mongodb("mongodb://localhost:27017/teotestintegration");
        g.reset_database();
        g.r#enum("Sex", |e| {
            e.localized_name("性别");
            e.description("性别，多用于用户和管理员。");
            e.choice("MALE", |c| {
                c.localized_name("男");
            });
            e.choice("FEMALE", |c| {
                c.localized_name("女");
            });
        });

        g.model("Simple", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("uniqueString", |f| {
                f.unique().required().string();
            });
            m.field("requiredString", |f| {
                f.required().string();
            });
            m.field("optionalString", |f| {
                f.optional().string();
            });
            m.field("optionalEnum", |f| {
                f.optional().r#enum("Sex");
            });
            m.field("requiredWithDefault", |f| {
                f.required().i8().default(2);
            });
            m.field("readonly", |f| {
                f.readonly().required().bool().default(true);
            });
            m.field("writeonly", |f| {
                f.writeonly().required().bool().default(false);
            });
        });
        g.model("Compound", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("one", |f| {
                f.required().string();
            });
            m.field("two", |f| {
                f.required().string();
            });
            m.field("three", |f| {
                f.required().string();
            });
            m.unique(vec!["one", "two"]);
        });
        g.model("List", |m| {
            m.field("id", |f| {
                f.primary().required().readonly().object_id().column_name("_id").auto();
            });
            m.field("listOne", |f| {
                f.required().vec(|f| {
                    f.string().on_save(|p| {
                        p.str_append("-suffix");
                    });
                });
            });
        });
    }).await
}

async fn app() -> App<impl ServiceFactory<
    ServiceRequest,
    Response = ServiceResponse<BoxBody>,
    Config = (),
    InitError = (),
    Error = Error,
>> {
    let graph = make_mongodb_graph().await;
    make_app(graph, ServerConfiguration::default())
}

#[test]
#[serial]
async fn create_with_valid_data_creates_entry() {
    let app = test::init_service(app().await).await;
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1"
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("uniqueString").unwrap(), &Value::String("1".to_string()));
    assert_eq!(body_data.get("requiredString").unwrap(), &Value::String("1".to_string()));
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str))
}

#[test]
#[serial]
async fn create_with_required_field_omitted_cannot_create() {
    let app = test::init_service(app().await).await;
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("data"), None);
    let body_error = body_obj.get("error").unwrap().as_hashmap().unwrap();
    assert_eq!(body_error.get("type").unwrap().as_str().unwrap(), "ValidationError");
    assert_eq!(body_error.get("message").unwrap().as_str().unwrap(), "Value is required.");
    let body_error_errors = body_error.get("errors").unwrap();
    assert_eq!(body_error_errors, &tson!({
        "requiredString": "Value is required."
    }));
}

#[test]
#[serial]
async fn create_with_duplicated_unique_value_cannot_create() {
    let app = test::init_service(app().await).await;
    let req1 = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1"
        }
    })).to_request();
    let req2 = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1"
        }
    })).to_request();
    let _: ServiceResponse = test::call_service(&app, req1).await;
    let resp: ServiceResponse = test::call_service(&app, req2).await;
    assert!(resp.status().is_client_error());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("data"), None);
    let body_error = body_obj.get("error").unwrap().as_hashmap().unwrap();
    assert_eq!(body_error.get("type").unwrap().as_str().unwrap(), "ValidationError");
    assert_eq!(body_error.get("message").unwrap().as_str().unwrap(), "Input is not valid.");
    let body_error_errors = body_error.get("errors").unwrap();
    assert_eq!(body_error_errors, &tson!({
        "uniqueString": "Unique value duplicated."
    }));
}

#[test]
#[serial]
async fn create_with_optional_data_creates_entry() {
    let app = test::init_service(app().await).await;
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1",
            "optionalString": "1"
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("uniqueString").unwrap(), &Value::String("1".to_string()));
    assert_eq!(body_data.get("requiredString").unwrap(), &Value::String("1".to_string()));
    assert_eq!(body_data.get("optionalString").unwrap(), &Value::String("1".to_string()));
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str))
}

#[test]
#[serial]
async fn create_with_correct_enum_value_creates_entry() {
    let app = test::init_service(app().await).await;
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1",
            "optionalEnum": "MALE"
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("uniqueString").unwrap(), &Value::String("1".to_string()));
    assert_eq!(body_data.get("requiredString").unwrap(), &Value::String("1".to_string()));
    assert_eq!(body_data.get("optionalEnum").unwrap(), &Value::String("MALE".to_string()));
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str))
}

#[test]
#[serial]
async fn create_with_invalid_enum_choice_value_cannot_create() {
    let app = test::init_service(app().await).await;
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1",
            "optionalEnum": "PUCK"
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("data"), None);
    let body_error = body_obj.get("error").unwrap().as_hashmap().unwrap();
    assert_eq!(body_error.get("type").unwrap().as_str().unwrap(), "ValidationError");
    assert_eq!(body_error.get("message").unwrap().as_str().unwrap(), "Enum value is unexpected.");
    let body_error_errors = body_error.get("errors").unwrap();
    assert_eq!(body_error_errors, &tson!({
        "optionalEnum": "Enum value is unexpected."
    }));
}

#[test]
#[serial]
async fn create_with_required_omitted_but_default_can_create() {
    let app = test::init_service(app().await).await;
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1"
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("requiredWithDefault").unwrap(), &Value::Number(Number::from(2)));
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str))
}

#[test]
#[serial]
async fn create_default_field_use_provided_value_if_exists() {
    let app = test::init_service(app().await).await;
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1",
            "requiredWithDefault": 8
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("requiredWithDefault").unwrap(), &Value::Number(Number::from(8)));
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str))
}

#[test]
#[serial]
async fn create_cannot_accept_readonly_value() {
    let app = test::init_service(app().await).await;
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1",
            "readonly": false,
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("data"), None);
    let body_error = body_obj.get("error").unwrap().as_hashmap().unwrap();
    assert_eq!(body_error.get("type").unwrap().as_str().unwrap(), "KeysUnallowed");
    assert_eq!(body_error.get("message").unwrap().as_str().unwrap(), "Unallowed keys detected.");
}

#[test]
#[serial]
async fn wont_output_writeonly_value() {
    let app = test::init_service(app().await).await;
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "2",
            "requiredString": "2",
            "writeonly": true,
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("error"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("writeonly"), None);
}

#[test]
#[serial]
async fn find_unique_can_find_by_primary_key() {
    let app = test::init_service(app().await).await;
    let create_req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1",
        }
    })).to_request();
    let create_resp: ServiceResponse = test::call_service(&app, create_req).await;
    let create_body_json: Value = test::read_body_json(create_resp).await;
    let id = create_body_json.as_hashmap().unwrap().get("data").unwrap().as_hashmap().unwrap().get("id").unwrap().as_str().unwrap();
    let find_unique_req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "FindUnique",
        "where": {
            "id": id
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, find_unique_req).await;
    //assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("uniqueString").unwrap(), &Value::String("1".to_string()));
    assert_eq!(body_data.get("requiredString").unwrap(), &Value::String("1".to_string()));
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str))
}

#[test]
#[serial]
async fn find_unique_can_find_by_single_unique_key() {
    let app = test::init_service(app().await).await;
    let create_req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1",
        }
    })).to_request();
    let _: ServiceResponse = test::call_service(&app, create_req).await;
    let find_unique_req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "FindUnique",
        "where": {
            "uniqueString": "1"
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, find_unique_req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("uniqueString").unwrap(), &Value::String("1".to_string()));
    assert_eq!(body_data.get("requiredString").unwrap(), &Value::String("1".to_string()));
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str))
}

#[test]
#[serial]
async fn find_unique_can_find_by_compound_unique_key() {
    let app = test::init_service(app().await).await;
    let create_req = test::TestRequest::post().uri("/compounds/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "one": "1",
            "two": "2",
            "three": "3",
        }
    })).to_request();
    let _: ServiceResponse = test::call_service(&app, create_req).await;
    let find_unique_req = test::TestRequest::post().uri("/compounds/action").set_tson(tson!({
        "action": "FindUnique",
        "where": {
            "one": "1",
            "two": "2"
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, find_unique_req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("one").unwrap(), &Value::String("1".to_string()));
    assert_eq!(body_data.get("two").unwrap(), &Value::String("2".to_string()));
    assert_eq!(body_data.get("three").unwrap(), &Value::String("3".to_string()));
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str));
}

#[test]
#[serial]
async fn find_many_can_find_all() {
    let app = test::init_service(app().await).await;
    let create_req = test::TestRequest::post().uri("/compounds/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "one": "1",
            "two": "2",
            "three": "3",
        }
    })).to_request();
    let _: ServiceResponse = test::call_service(&app, create_req).await;
    let create_req = test::TestRequest::post().uri("/compounds/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "one": "one",
            "two": "two",
            "three": "three",
        }
    })).to_request();
    let _: ServiceResponse = test::call_service(&app, create_req).await;
    let find_many_req = test::TestRequest::post().uri("/compounds/action").set_tson(tson!({
        "action": "FindMany"
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, find_many_req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta").unwrap(), &tson!({"count": 2}));
    assert_eq!(body_obj.get("errors"), None);
    let body_array = body_obj.get("data").unwrap().as_vec().unwrap();
    let body_data_1 = body_array.get(0).unwrap().as_hashmap().unwrap();
    assert_eq!(body_data_1.get("one").unwrap(), &Value::String("1".to_string()));
    assert_eq!(body_data_1.get("two").unwrap(), &Value::String("2".to_string()));
    assert_eq!(body_data_1.get("three").unwrap(), &Value::String("3".to_string()));
    assert!(is_object_id(body_data_1.get("id").unwrap().as_str().unwrap()));
    let body_data_2 = body_array.get(1).unwrap().as_hashmap().unwrap();
    assert_eq!(body_data_2.get("one").unwrap(), &Value::String("one".to_string()));
    assert_eq!(body_data_2.get("two").unwrap(), &Value::String("two".to_string()));
    assert_eq!(body_data_2.get("three").unwrap(), &Value::String("three".to_string()));
    assert!(is_object_id(body_data_2.get("id").unwrap().as_str().unwrap()));
}

#[test]
#[serial]
async fn find_many_can_find_all_filtered_by_where() {
    let app = test::init_service(app().await).await;
    let create_req = test::TestRequest::post().uri("/compounds/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "one": "1",
            "two": "2",
            "three": "3",
        }
    })).to_request();
    let _: ServiceResponse = test::call_service(&app, create_req).await;
    let create_req = test::TestRequest::post().uri("/compounds/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "one": "one",
            "two": "two",
            "three": "three",
        }
    })).to_request();
    let _: ServiceResponse = test::call_service(&app, create_req).await;
    let find_many_req = test::TestRequest::post().uri("/compounds/action").set_tson(tson!({
        "action": "FindMany",
        "where": {
            "one": {
                "equals": "one"
            }
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, find_many_req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta").unwrap(), &tson!({"count": 1}));
    assert_eq!(body_obj.get("errors"), None);
    let body_array = body_obj.get("data").unwrap().as_vec().unwrap();
    let body_data_2 = body_array.get(0).unwrap().as_hashmap().unwrap();
    assert_eq!(body_data_2.get("one").unwrap(), &Value::String("one".to_string()));
    assert_eq!(body_data_2.get("two").unwrap(), &Value::String("two".to_string()));
    assert_eq!(body_data_2.get("three").unwrap(), &Value::String("three".to_string()));
    assert!(is_object_id(body_data_2.get("id").unwrap().as_str().unwrap()));
}

#[test]
#[serial]
async fn update_can_update_valid_contents() {
    let app = test::init_service(app().await).await;
    let create_req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1"
        }
    })).to_request();
    let create_resp: ServiceResponse = test::call_service(&app, create_req).await;
    let create_body_json: Value = test::read_body_json(create_resp).await;
    let create_body_obj = create_body_json.as_hashmap().unwrap();
    let id = create_body_obj.get("data").unwrap().as_hashmap().unwrap().get("id").unwrap().as_str().unwrap();
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Update",
        "where": {
            "id": id
        },
        "update": {
            "uniqueString": "5",
            "requiredString": "5"
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("uniqueString").unwrap(), &Value::String("5".to_string()));
    assert_eq!(body_data.get("requiredString").unwrap(), &Value::String("5".to_string()));
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str))
}

#[test]
#[serial]
async fn update_can_set_optional_value_back_to_null() {
    let app = test::init_service(app().await).await;
    let create_req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1",
            "optionalString": "5"
        }
    })).to_request();
    let create_resp: ServiceResponse = test::call_service(&app, create_req).await;
    let create_body_json: Value = test::read_body_json(create_resp).await;
    let create_body_obj = create_body_json.as_hashmap().unwrap();
    let id = create_body_obj.get("data").unwrap().as_hashmap().unwrap().get("id").unwrap().as_str().unwrap();
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Update",
        "where": {
            "id": id
        },
        "update": {
            "uniqueString": "5",
            "requiredString": "5",
            "optionalString": null
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("uniqueString").unwrap(), &Value::String("5".to_string()));
    assert_eq!(body_data.get("requiredString").unwrap(), &Value::String("5".to_string()));
    assert_eq!(body_data.get("optionalString"), None);
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str))
}

#[test]
#[serial]
async fn delete_can_delete_record() {
    let app = test::init_service(app().await).await;
    let create_req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "uniqueString": "1",
            "requiredString": "1"
        }
    })).to_request();
    let create_resp: ServiceResponse = test::call_service(&app, create_req).await;
    let create_body_json: Value = test::read_body_json(create_resp).await;
    let create_body_obj = create_body_json.as_hashmap().unwrap();
    let id = create_body_obj.get("data").unwrap().as_hashmap().unwrap().get("id").unwrap().as_str().unwrap();
    let req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "Delete",
        "where": {
            "id": id
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("uniqueString").unwrap(), &Value::String("1".to_string()));
    assert_eq!(body_data.get("requiredString").unwrap(), &Value::String("1".to_string()));
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str));
    // now find many
    let find_many_req = test::TestRequest::post().uri("/simples/action").set_tson(tson!({
        "action": "FindMany"
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, find_many_req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta").unwrap(), &tson!({"count": 0}));
}

#[test]
#[serial]
async fn create_vec_works_with_inner_pipeline() {
    let app = test::init_service(app().await).await;
    let req = test::TestRequest::post().uri("/lists/action").set_tson(tson!({
        "action": "Create",
        "create": {
            "listOne": ["1", "2"],
        }
    })).to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body_json: Value = test::read_body_json(resp).await;
    let body_obj = body_json.as_hashmap().unwrap();
    assert_eq!(body_obj.get("meta"), None);
    assert_eq!(body_obj.get("errors"), None);
    let body_data = body_obj.get("data").unwrap().as_hashmap().unwrap();
    assert_eq!(body_data.get("listOne").unwrap(), &tson!([
        "1-suffix", "2-suffix"
    ]));
    let id_str = body_data.get("id").unwrap().as_str().unwrap();
    assert!(is_object_id(id_str))
}
