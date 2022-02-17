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

use serde::{de, Deserialize, Deserializer, Serialize};
use futures::executor;

use actix_web::{rt::System, web, get, App, HttpRequest, HttpResponse, HttpServer, Responder, middleware::Logger};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslVerifyMode, SslOptions, SslMode};
use rand::{distributions::Alphanumeric, Rng};

use prost::Message;

use mhycrypt;
use pretty_env_logger::env_logger::fmt;
use serde::de::Unexpected;
//use openssl::rand;

#[derive(Clone)]
pub struct DispatchServer {}

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
/*
#[derive(Deserialize,Debug)]
struct GranterData {
    app_id: String,
    channel_id: String,
    device: String,
    sign: String,
    data: String,
}*/

#[derive(Deserialize,Debug)]
struct GranterData {
    #[serde(deserialize_with = "deserialize_u32_or_string")]
    app_id: u32,
    #[serde(deserialize_with = "deserialize_u32_or_string")]
    channel_id: u32,
    device: String,
    sign: String,
    data: String,
}

/* Deserialization hack */
fn deserialize_u32_or_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StrOrU32<'a> {
        Str(&'a str),
        U32(u32),
    }

    Ok(match StrOrU32::deserialize(deserializer)? {
        StrOrU32::Str(v) => v.parse().unwrap(), // Ignoring parsing errors
        StrOrU32::U32(v) => v,
    })
}

#[derive(Deserialize,Debug)]
struct MinorApiLogData {
    data: String,
}

#[derive(Deserialize,Debug)]
struct GeetestGetData {
    gt: String,
    challenge: String,
    lang: String,
    is_next: Option<bool>,
    client_type: Option<String>,
    w: Option<String>,
    pt: Option<u32>,
    callback: Option<String>,
}

#[derive(Deserialize,Debug)]
struct GeetestGetTypeData {
    gt: String,
    t: u64,
    callback: Option<String>,
}

#[derive(Deserialize,Debug)]
struct GeetestAjaxData {
    gt: String,
    challenge: String,
    client_type: Option<String>,
    w: Option<String>,
    callback: Option<String>,
    #[serde(rename = "$_BBF")]
    BBF: Option<u32>,
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
        println!("Hostname {}, local IP {}", DispatchServer::get_hostname(), DispatchServer::get_local_ip());

        let (http_port, https_port) = (80, 443);

        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls_server()).unwrap();
        //builder.set_verify(SslVerifyMode::NONE);
        //builder.set_min_proto_version(None).unwrap();
        //builder.set_cipher_list("DEFAULT").unwrap();
        //builder.set_mode(SslMode::NO_AUTO_CHAIN | SslMode::SEND_FALLBACK_SCSV);
        builder.set_private_key_file("keys/ssl.key", SslFiletype::PEM).unwrap();
        builder.set_certificate_chain_file("keys/ssl.cer").unwrap();

        let http_server = HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .route("/", web::get().to(|| HttpResponse::Ok()))
                .route("/query_security_file", web::get().to(DispatchServer::query_security_file))
                .route("/query_region_list", web::get().to(DispatchServer::query_region_list))
                .route("/query_cur_region", web::get().to(DispatchServer::query_cur_region))
                //.route("", web::post().to(DispatchServer::))
                .route("/hk4e_global/mdk/shield/api/verify", web::post().to(DispatchServer::shield_verify))
                //.route("/account/risky/api/check", web::post().to(DispatchServer::risky_api_check))
                .route("/account/risky/api/check", web::post().to(DispatchServer::risky_api_check_old))
                .route("/hk4e_global/mdk/shield/api/login", web::post().to(DispatchServer::shield_login))
                .route("/hk4e_global/combo/granter/login/v2/login", web::post().to(DispatchServer::granter_login))
                // Misc stuff, not really required
                .route("/common/h5log/log/batch", web::post().to(DispatchServer::minor_api_log))

                .route("/combo/box/api/config/sdk/combo", web::get().to(DispatchServer::combo_combo))
                .route("/hk4e_global/combo/granter/api/getConfig", web::get().to(DispatchServer::get_config))
                .route("/hk4e_global/mdk/shield/api/loadConfig", web::get().to(DispatchServer::load_config))
                //.route("/hk4e_global/combo/granter/api/getFont", web::get().to(DispatchServer::get_font))
                .route("/hk4e_global/mdk/agreement/api/getAgreementInfos", web::get().to(DispatchServer::get_agreement_infos))
                .route("/admin/mi18n/plat_oversea/m2020030410/m2020030410-version.json", web::get().to(DispatchServer::version_data))
                .route("/hk4e_global/combo/granter/api/compareProtocolVersion", web::post().to(DispatchServer::compare_protocol_version))
                // GEETEST
                .route("/get.php", web::get().to(DispatchServer::geetest_get))
                .route("/gettype.php", web::get().to(DispatchServer::geetest_get_type))
                .route("/ajax.php", web::get().to(DispatchServer::geetest_ajax_get))
                .route("/ajax.php", web::post().to(DispatchServer::geetest_ajax_post))
                // Logging
                .route("/log/sdk/upload", web::post().to(DispatchServer::log_skip))
                .route("/sdk/dataUpload", web::post().to(DispatchServer::log_skip))
                .route("/crash/dataUpload", web::post().to(DispatchServer::log_skip))

        })
        .bind(format!("0.0.0.0:{}", http_port)).expect("Failed to bind HTTP port")
        .bind_openssl(format!("0.0.0.0:{}", https_port), builder).expect("Failed to bind HTTPS port")
        .run();

        http_server.stop(true).await;
    }

    async fn query_security_file() -> String {
        return "".to_string();
    }

    async fn query_region_list(c: web::Query<ClientInfo>) -> String {
        println!("RegionList, Client: {:?}", c);

        let keys = DispatchServer::load_keys("master");
        
        let mut region_info = proto::RegionSimpleInfo::default();
        region_info.name = "ps_rusty".into();
        region_info.title = "Rusty Samovar".into();
        region_info.r#type = "DEV_PUBLIC".into();
        //region_info.dispatch_url = format!("http://{}:{}/query_cur_region", DispatchServer::get_hostname(), 80);
        region_info.dispatch_url = format!("http://127.0.0.1:{}/query_cur_region", 80);

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
        println!("CurRegion, Client: {:?}", c);

        let keys = DispatchServer::load_keys("master");
        
        let mut region_info = proto::RegionInfo::default();
        region_info.gateserver_ip = DispatchServer::get_local_ip();
        region_info.gateserver_port = 4242;
        region_info.secret_key = keys.0.clone();

        let mut region_config = proto::QueryCurrRegionHttpRsp::default();
        region_config.region_info = Some(region_info);
        region_config.client_secret_key = keys.0.clone();

        let json_config = format!("{{\"coverSwitch\": [\"8\"], \"perf_report_config_url\": \"http://{}:{}/config/verify\", \"perf_report_record_url\": \"http://{}:{}/dataUpload\" }}",
            DispatchServer::get_hostname(), 80, DispatchServer::get_hostname(), 80);

        let mut custom_config = json_config.as_bytes().to_owned();

        mhycrypt::mhy_xor(&mut custom_config, &keys.1);

        region_config.region_custom_config_encrypted = custom_config.to_vec();

        let mut region_conf_buf = Vec::new();

        region_config.encode(&mut region_conf_buf).unwrap();

        return base64::encode(region_conf_buf);
    }

    async fn risky_api_check_old(a: web::Json<ActionToCheck>) -> String {
        println!("Action: {:?}", a);

        let email = "ceo@hoyolab.com";
        let name = "Ceo";
        let token = Self::generate_fake_token();
        let uid = 0x1234;

        let payload = DispatchServer::build_account_data(email, name, &token, uid);

        return DispatchServer::make_answer(0, &payload);
    }

    async fn risky_api_check(a: web::Json<ActionToCheck>) -> String {
        println!("Action: {:?}", a);

        let challenge = "5876e8bb6d90e0d6cf4dd26b109fe508";
        let gt = "16bddce04c7385dbb7282778c29bba3e";
        let id = "a0f5968aa4664b55ac914bffa1cd8058";

        let payload = format!("
            {{
                \"action\": \"ACTION_GEETEST\",
                \"geetest\": {{
                    \"challenge\": \"{}\",
                    \"gt\": \"{}\",
                    \"new_captcha\": 1,
                    \"success\": 1
                }},
                \"id\": \"{}\"
            }}
        ", challenge, gt, id);

        return DispatchServer::make_answer(0, &payload);
    }

    async fn shield_login(l: web::Json<LoginData>) -> String {
        println!("Login: {:?}", l);

        let email = "ceo@hoyolab.com";
        let name = "Ceo";
        let token = Self::generate_fake_token();
        let uid = 0x1234;

        let payload = DispatchServer::build_account_data(email, name, &token, uid);

        return DispatchServer::make_answer(0, &payload);
    }

    async fn granter_login(g: web::Json<GranterData>) -> String {
        println!("Granter: {:?}", g);

        let payload = DispatchServer::verify_token_v2();

        return DispatchServer::make_answer(0, &payload);
    }

    async fn combo_combo() -> String {

        let payload = format!("{{
            \"vals\": {{
                \"disable_email_bind_skip\": \"false\",
                \"email_bind_remind\": \"true\",
                \"email_bind_remind_interval\": \"7\"
            }}
        }}");

        return DispatchServer::make_answer(0,&payload);
    }

    async fn get_config() -> String {
        let payload = format!("{{
            \"announce_url\": \"https://localhost/hk4e/announcement/index.html\",
            \"disable_ysdk_guard\": false,
            \"enable_announce_pic_popup\": true,
            \"log_level\": \"INFO\",
            \"protocol\": true,
            \"push_alias_type\": 2,
            \"qr_enabled\": false
        }}");

        return DispatchServer::make_answer(0,&payload);
    }

    async fn load_config() -> String {
        let payload = format!("{{
            \"client\": \"PC\",
            \"disable_mmt\": false,
            \"disable_regist\": false,
            \"enable_email_captcha\": false,
            \"enable_ps_bind_account\": false,
            \"game_key\": \"hk4e_global\",
            \"guest\": false,
            \"id\": 6,
            \"identity\": \"I_IDENTITY\",
            \"ignore_versions\": \"\",
            \"name\": \"原神海外\",
            \"scene\": \"S_NORMAL\",
            \"server_guest\": false,
            \"thirdparty\": [
                \"fb\",
                \"tw\"
            ],
            \"thirdparty_ignore\": {{
                \"fb\": \"\",
                \"tw\": \"\"
            }}
        }}");
        return DispatchServer::make_answer(0,&payload);
    }

    async fn shield_verify(t: web::Json<TokenToVerify>) -> String {
        println!("Token: {:?}", t);

        let email = "ceo@hoyolab.com";
        let name = "Ceo";
        let token = t.token.clone();
        let uid = t.uid.parse().unwrap();

        let payload = DispatchServer::build_account_data(email, name, &token, uid);

        return DispatchServer::make_answer(0, &payload);
    }

    async fn minor_api_log(l: web::Json<MinorApiLogData>) -> String {
        return "{\"retcode\":0,\"message\":\"success\",\"data\":null}".to_string();
    }

    /*
        GEETEST
     */
    async fn geetest_get(g: web::Query<GeetestGetData>) -> String {
        println!("GeetestGet: {:?}", g);

        let is_next = match g.is_next {
            None => false,
            Some(_) => true,
        };

        if (is_next) {
            let callback = g.callback.as_ref().unwrap();
            
            return format!("
                {}( {{
                    \"gt\": \"{}\",
                    \"challenge\": \"{}\",
                    \"id\": \"a7b56e21f6771ab10e2bc4a3a511c4be0\", 
                    \"bg\": \"pictures/gt/1dce8a0cd/bg/744f986a0.jpg\", 
                    \"fullbg\": \"pictures/gt/1dce8a0cd/1dce8a0cd.jpg\", 
                    \"link\": \"\",
                    \"ypos\": 85,
                    \"xpos\": 0,
                    \"height\": 160,
                    \"slice\": \"pictures/gt/1dce8a0cd/slice/744f986a0.png\", \
                    \"api_server\": \"https://api-na.geetest.com/\",
                    \"static_servers\": [\"static.geetest.com/\", \"dn-staticdown.qbox.me/\"],
                    \"mobile\": true,
                    \"theme\": \"ant\",
                    \"theme_version\": \"1.2.6\",
                    \"template\": \"\",
                    \"logo\": false,
                    \"clean\": false,
                    \"type\": \"multilink\",
                    \"fullpage\": false,
                    \"feedback\": \"\",
                    \"show_delay\": 250,
                    \"hide_delay\": 800,
                    \"benchmark\": false,
                    \"version\": \"6.0.9\",
                    \"product\": \"embed\",
                    \"https\": true,
                    \"width\": \"100%\",
                    \"c\": [12, 58, 98, 36, 43, 95, 62, 15, 12],
                    \"s\": \"6b70592c\",
                    \"so\": 0,
                    \"i18n_labels\": {{
                        \"cancel\": \"Cancel\",
                        \"close\": \"Close\",
                        \"error\": \"Error. Close and retry.\",
                        \"fail\": \"Incorrect position\",
                        \"feedback\": \"Info\",
                        \"forbidden\": \"Retry after 3 seconds\",
                        \"loading\": \"Loading\",
                        \"logo\": \"Geetest\",
                        \"read_reversed\": false,
                        \"refresh\": \"Refresh\",
                        \"slide\": \"Slide to unlock\",
                        \"success\": \"sec s. You're better than score% of users\",
                        \"tip\": \"\",
                        \"voice\": \"Voice test\"
                    }},
                    \"gct_path\": \"/static/js/gct.d0a2919ae56f007ecb8e22fb47f80f33.js\"
                }} )", callback, g.gt, g.challenge);
        } else {
            let data = "
                ( {
                    \"status\": \"success\",
                    \"data\": {
                        \"theme\": \"wind\",
                        \"theme_version\": \"1.5.8\",
                        \"static_servers\": [\"static.geetest.com\", \"dn-staticdown.qbox.me\"],
                        \"api_server\": \"api-na.geetest.com\",
                        \"logo\": false,
                        \"feedback\": \"\",
                        \"c\": [12, 58, 98, 36, 43, 95, 62, 15, 12],
                        \"s\": \"3f6b3542\",
                        \"i18n_labels\": {
                            \"copyright\": \"Geetest\",
                            \"error\": \"Error\",
                            \"error_content\": \"Retry\",
                            \"error_title\": \"Timeout\",
                            \"fullpage\": \"Confirm\",
                            \"goto_cancel\": \"Cancel\",
                            \"goto_confirm\": \"OK\",
                            \"goto_homepage\": \"Go to Geetest homepage?\",
                            \"loading_content\": \"Confirm\",
                            \"next\": \"Loaging\",
                            \"next_ready\": \"Not fulfilled\",
                            \"read_reversed\": false,
                            \"ready\": \"Click to confirm\",
                            \"refresh_page\": \"Error. Refresh the page to continue.\",
                            \"reset\": \"Retry\",
                            \"success\": \"Success\",
                            \"success_title\": \"Success\"
                        }
                    }
                })
            ";

            return match g.callback.as_ref() {
                None => data.to_string(),
                Some(callback) => format!(
                    "{}{}",
                    callback, data),
            }
        }
    }

    async fn geetest_get_type(gt: web::Query<GeetestGetTypeData>) -> String {
        println!("GeetestGetType: {:?}", gt);

        let data = "\
            ( {
                \"status\": \"success\",
                \"data\": {
                    \"type\": \"fullpage\",
                    \"static_servers\": [\"static.geetest.com/\", \"dn-staticdown.qbox.me/\"],
                    \"click\": \"/static/js/click.3.0.2.js\",
                    \"pencil\": \"/static/js/pencil.1.0.3.js\",
                    \"voice\": \"/static/js/voice.1.2.0.js\",
                    \"fullpage\": \"/static/js/fullpage.9.0.8.js\",
                    \"beeline\": \"/static/js/beeline.1.0.1.js\",
                    \"slide\": \"/static/js/slide.7.8.6.js\",
                    \"geetest\": \"/static/js/geetest.6.0.9.js\",
                    \"aspect_radio\": {
                        \"slide\": 103, \"click\": 128, \"voice\": 128, \"pencil\": 128, \"beeline\": 50
                    }
                }
            })
        ";

        return match &gt.callback {
            None => data.to_string(),
            Some(callback) => format!(
                "{}{}",
            callback, data),
        };
    }

    async fn geetest_ajax_get(ga: web::Query<GeetestAjaxData>) -> String {
        return Self::geetest_ajax(ga.into_inner()).await;
    }

    async fn geetest_ajax_post(ga: web::Json<GeetestAjaxData>) -> String {
        return Self::geetest_ajax(ga.into_inner()).await;
    }

    async fn geetest_ajax(ga: GeetestAjaxData) -> String {
        println!("GeetestAjax: {:?}", ga);

        let is_next = match ga.BBF {
            None => false,
            Some(_) => true,
        };

        if (is_next) {
            let callback = ga.callback.as_ref().unwrap();

            return format!("
                {}( {{
                \"success\": 1,
                \"message\": \"success\",
                \"validate\": \"\",
                \"score\": \"11\"
            }} )", callback);
        } else {
            let data = "
                {
                    \"status\": \"success\",
                    \"data\": {
                        \"result\": \"slide\"
                    }
                }
            ";

            return match ga.callback.as_ref() {
                None => data.to_string(),
                Some(callback) => format!(
                    "{}(
                        {}
                    )",
                callback, data),
            }
        }
    }

    async fn log_skip(body: web::Bytes) -> String {
        //println!("Logging: {}", std::str::from_utf8(&body).unwrap());

        return "{}".to_string();
    }

    async fn get_agreement_infos() -> String {
        let payload = format!("{{
            \"marketing_agreements\": []
        }}");

        return DispatchServer::make_answer(0, &payload);
    }

    async fn compare_protocol_version() -> String {
        let payload = format!("{{
            \"modified\": true,
            \"protocol\": {{
                \"app_id\": 4,
                \"create_time\": \"0\",
                \"id\": 0,
                \"language\": \"ru\",
                \"major\": 4,
                \"minimum\": 0,
                \"priv_proto\": \"\",
                \"teenager_proto\": \"\",
                \"user_proto\": \"\"
            }}
        }}");

        return DispatchServer::make_answer(0, &payload);
    }

    async fn version_data() -> String {
        return "{\"version\": 54}".to_string();
    }

    fn get_hostname() -> String {
        return hostname::get().unwrap().into_string().unwrap();
        //return "localhost";
    }

    fn get_local_ip() -> String {
        //return local_ip_address::local_ip().unwrap().to_string();
        return "127.0.0.1".to_string();
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
        let combo_token = Self::generate_fake_token();
        #[cfg(feature = "raw_packet_dump")]
        let combo_token = std::str::from_utf8(&[32u8; 4096*3]).unwrap();

        return format!("{{
            \"account_type\": \"{}\",
            \"combo_id\": \"{}\",
            \"combo_token\": \"{}\",
            \"data\": {{\"guest\": \"false\"}},
            \"heartbeat\": false,
            \"open_id\": \"{}\"
        }}", account_type, combo_id, combo_token, open_id);
    }

    fn build_account_data(email: &str, name: &str, token: &str, uid: i32) -> String {
        let payload = format!("{{
                \"account\": {{
                    \"apple_name\": \"\",
                    \"area_code\": \"**\",
                    \"country\": \"US\",
                    \"device_grant_ticket\": \"\",
                    \"email\": \"{}\",
                    \"facebook_name\": \"\",
                    \"game_center_name\": \"\",
                    \"google_name\": \"\",
                    \"identity_card\": \"\",
                    \"is_email_verify\": \"0\",
                    \"mobile\": \"\",
                    \"name\": \"{}\",
                    \"reactivate_ticket\": \"\",
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
                \"realname_operation\": \"None\",
                \"realperson_required\": false,
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

    fn generate_fake_token() -> String {
        return rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
    }
}
