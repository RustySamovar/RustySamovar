use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

extern crate futures;
extern crate base64;
extern crate actix_web;
extern crate openssl;

use serde::{Deserialize,Serialize};
use futures::executor;

use actix_web::{rt::System, web, get, App, HttpRequest, HttpResponse, HttpServer, Responder, middleware::Logger};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslVerifyMode, SslOptions, SslMode};

use prost::Message;

use mhycrypt;

#[derive(Clone)]
pub struct DispatchServer {
}

#[derive(Deserialize,Debug)]
struct ClientInfo {
    version: String,
    lang: i32,
    platform: i32,
    binary: i32,
    time: i32,
    channel_id: i32,
    sub_channel_id: i32,
    account_type: Option<i32>,
}

#[derive(Deserialize,Debug)]
struct TokenToVerify
{
    uid: String,
    token: String,
}

#[derive(Deserialize,Debug)]
struct ActionToCheck
{
    action_type: String,
    api_name: String,
    username: Option<String>,
}

#[derive(Deserialize,Debug)]
struct LoginData {
    account: String,
    is_crypto: bool,
    password: String,
}

#[derive(Deserialize,Debug)]
struct GranterData {
    app_id: String,
    channel_id: String,
    device: String,
    sign: String,
    data: String,
}

impl DispatchServer {
    pub fn new() -> DispatchServer {
        let ds = DispatchServer { };

        return ds;
    }

    pub fn run(self) {
        let mut sys = System::new("http-server");
        let slef = Arc::new(self);
        executor::block_on(slef.run_internal());
        System::current().stop();
        println!("Finished!");
    }

    async fn run_internal(self: &Arc<Self>) {
        //let (http_port, https_port) = (2880, 2443);
        let (http_port, https_port) = (80, 443);

        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls_server()).unwrap();
        builder.set_verify(SslVerifyMode::NONE);
        builder.set_min_proto_version(None).unwrap();
        builder.set_cipher_list("DEFAULT").unwrap();
        builder.set_mode(SslMode::NO_AUTO_CHAIN | SslMode::SEND_FALLBACK_SCSV);
        builder.set_private_key_file("keys/ssl.key", SslFiletype::PEM).unwrap();
        builder.set_certificate_chain_file("keys/ssl.cer").unwrap();

        let http_server = HttpServer::new(move || {
            App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(|| HttpResponse::Ok()))
            .route("/query_region_list", web::get().to(DispatchServer::query_region_list))
            .route("/query_cur_region", web::get().to(DispatchServer::query_cur_region))
            //.route("", web::post().to(DispatchServer::))
            .route("/hk4e_global/mdk/shield/api/verify", web::post().to(DispatchServer::shield_verify))
            .route("/account/risky/api/check", web::post().to(DispatchServer::risky_api_check))
            .route("/hk4e_global/mdk/shield/api/login", web::post().to(DispatchServer::shield_login))
            .route("/hk4e_global/combo/granter/login/v2/login", web::post().to(DispatchServer::granter_login))
        })
        .bind(format!("127.0.0.1:{}", http_port)).expect("Failed to bind HTTP port")
        .bind_openssl(format!("127.0.0.1:{}", https_port), builder).expect("Failed to bind HTTPS port")
        .run();

        http_server.stop(true).await;
    }

    async fn query_region_list(c: web::Query<ClientInfo>) -> String {
        println!("Client: {:?}", c);

        let keys = DispatchServer::load_keys("master");
        
        let mut region_info = proto::RegionSimpleInfo::default();
        region_info.name = "private_server".into();
        region_info.title = "Private Server".into();
        region_info.r#type = "DEV_PUBLIC".into();
        region_info.dispatch_url = format!("http://localhost:{}/query_cur_region", 80);

        let mut region_list = proto::QueryRegionListHttpRsp::default();
        region_list.region_list = vec![region_info];
        region_list.enable_login_pc = true;

        region_list.client_secret_key = keys.0.clone();

        let json_config = "{\"sdkenv\":\"2\",\"checkdevice\":\"false\",\"loadPatch\":\"false\",\"showexception\":\"false\",\"regionConfig\":\"pm|fk|add\",\"downloadMode\":\"0\"}";

        let mut custom_config = json_config.as_bytes().to_owned();

        mhycrypt::mhy_xor(&mut custom_config, &keys.1);

        region_list.client_custom_config_encrypted = custom_config.to_vec();

        let mut region_list_buf = Vec::new();

        region_list.encode(&mut region_list_buf).unwrap();

        return base64::encode(region_list_buf);
    }

    async fn query_cur_region(c: web::Query<ClientInfo>) -> String {
        println!("Client: {:?}", c);

        let keys = DispatchServer::load_keys("master");
        
        let mut region_info = proto::RegionInfo::default();
        region_info.gateserver_ip = "127.0.0.1".to_string();
        region_info.gateserver_port = 4242;
        region_info.secret_key = keys.0.clone();

        let mut region_config = proto::QueryCurrRegionHttpRsp::default();
        region_config.region_info = Some(region_info);
        region_config.client_secret_key = keys.0.clone();

        let json_config = format!("{{\"coverSwitch\": [\"8\"], \"perf_report_config_url\": \"http://localhost:{}/config/verify\", \"perf_report_record_url\": \"http://localhost:{}/dataUpload\" }}",
            80, 80);

        let mut custom_config = json_config.as_bytes().to_owned();

        mhycrypt::mhy_xor(&mut custom_config, &keys.1);

        region_config.region_custom_config_encrypted = custom_config.to_vec();

        let mut region_conf_buf = Vec::new();

        region_config.encode(&mut region_conf_buf).unwrap();

        return base64::encode(region_conf_buf);
    }

    async fn risky_api_check(a: web::Json<ActionToCheck>) -> String {
        println!("Action: {:?}", a);

        let email = "ceo@hoyolab.com";
        let name = "Ceo";
        let token = "Fake-token-hahaha";
        let uid = 0x1234;

        let payload = DispatchServer::build_account_data(email, name, token, uid);

        return DispatchServer::make_answer(0, &payload);
    }

    async fn shield_login(l: web::Json<LoginData>) -> String {
        println!("Login: {:?}", l);

        let email = "ceo@hoyolab.com";
        let name = "Ceo";
        let token = "Fake-token-hahaha";
        let uid = 0x1234;

        let payload = DispatchServer::build_account_data(email, name, token, uid);

        return DispatchServer::make_answer(0, &payload);
    }

    async fn granter_login(g: web::Json<GranterData>) -> String {
        println!("Granter: {:?}", g);

        let payload = DispatchServer::verify_token_v2();

        return DispatchServer::make_answer(0, &payload);
    }

    async fn shield_verify(t: web::Json<TokenToVerify>) -> String {
        println!("Token: {:?}", t);

        let email = "ceo@hoyolab.com";
        let name = "Ceo";
        let token = "Fake-token-hahaha";
        let uid = t.uid.parse().unwrap();

        let payload = DispatchServer::build_account_data(email, name, token, uid);

        return DispatchServer::make_answer(0, &payload);
    }

    fn load_keys(name: &str) -> (Vec<u8>, Vec<u8>) {
        // Key
        let filename = format!("./{}/{}.key", "keys", name);
        let mut f = fs::File::open(&filename).expect(&format!("File '{}' not found", filename));
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut key = vec![0; metadata.len() as usize];
        f.read(&mut key).expect("buffer overflow");
        // Ec2b
        let filename = format!("./{}/{}.ec2b", "keys", name);
        let mut f = fs::File::open(&filename).expect(&format!("File '{}' not found", filename));
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut ec2b = vec![0; metadata.len() as usize];
        f.read(&mut ec2b).expect("buffer overflow");
        return (ec2b, key);
    }

    fn verify_token_v2() -> String {
        let account_type = 1;
        let combo_id = 0x4321;
        let open_id = 0x1234;

        #[cfg(not(feature = "raw_packet_dump"))]
        let combo_token = "Fake-token-hehehe";
        #[cfg(feature = "raw_packet_dump")]
        let combo_token = std::str::from_utf8(&[32u8; 4096*3]).unwrap();

        return format!("{{
            \"account_type\": \"{}\",
            \"combo_id\": \"{}\",
            \"combo_token\": \"{}\",
            \"data\": {{\"guest\": \"false\"}},
            \"heartbeat\": \"false\",
            \"open_id\": \"{}\"
        }}", account_type, combo_id, combo_token, open_id);
    }

    fn build_account_data(email: &str, name: &str, token: &str, uid: i32) -> String {
        let payload = format!("{{
                \"account\": {{
                    \"apple_name\": \"\",
                    \"country\": \"\",
                    \"email\": \"{}\",
                    \"facebook_name\": \"\",
                    \"game_center_name\": \"\",
                    \"google_name\": \"\",
                    \"identity_card\": \"\",
                    \"is_email_verify\": \"0\",
                    \"mobile\": \"\",
                    \"name\": \"{}\",
                    \"realname\": \"\",
                    \"safe_mobile\": \"\",
                    \"sony_name\": \"\",
                    \"tap_name\": \"\",
                    \"token\": \"{}\",
                    \"twitter_name\": \"\",
                    \"uid\": \"{}\"
                }},
                \"device_grant_required\": \"false\",
                \"realperson_required\": \"false\",
                \"safe_moblie_required\": \"false\"
            }}", email, name, token, uid);

        return payload.into();
    }

    fn make_answer(code: i32, data: &str) -> String {
        let message = match code {
            0 => "OK",
            -1 => "not matched",
            _ => "ERROR",
        };

        return format!("{{
            \"retcode\": \"{}\",
            \"message\": \"{}\",
            \"data\": {}
        }}", code, message, data).to_string();
    }
}
