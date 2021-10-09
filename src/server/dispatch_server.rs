use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::collections::HashMap;
use std::fs;

use prost::Message;
extern crate base64;

use crate::proto;
use mhycrypt;

type RequestCallback = fn(slef: &DispatchServer, args: Vec<(&str, &str)>) -> (String, String);

pub struct DispatchServer {
    listener: TcpListener,
    callbacks: HashMap<String, RequestCallback>,
    ip: String,
    port: u16,
}

impl DispatchServer {
    pub fn new(ip: &str, port: u16) -> DispatchServer {
        let mut callbacks = HashMap::new();

        let mut ds = DispatchServer {
            listener: TcpListener::bind(format!("{}:{}", ip, port)).unwrap(),
            callbacks: callbacks,
            ip: ip.to_string(),
            port: port,
        };

        ds.callbacks.insert("/query_region_list".to_string(), DispatchServer::query_region_list);
        ds.callbacks.insert("/query_cur_region".to_string(), DispatchServer::query_cur_region);
        ds.callbacks.insert("/account/risky/api/check".to_string(), DispatchServer::risky_api_check);
        ds.callbacks.insert("/hk4e_global/mdk/shield/api/login".to_string(), DispatchServer::risky_api_check);
        ds.callbacks.insert("/hk4e_global/combo/granter/login/v2/login".to_string(), DispatchServer::granter_login);
        ds.callbacks.insert("/hk4e_global/mdk/shield/api/verify".to_string(), DispatchServer::shield_verify);

        return ds;
    }

    pub fn run(&self) {
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();

            self.handle_connection(stream);
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];

        stream.read(&mut buffer).unwrap();

        let buffer = String::from_utf8_lossy(&buffer);

        let data = buffer.split_whitespace().collect::<Vec<_>>();
        let method = data[0];
        let url = data[1];

        println!("Access to '{}' using {}", url, method);

        let (status_code, contents) = match method {
            "GET" | "POST" => { 
                let parts = url.split('?').collect::<Vec<_>>();
                let uri = parts[0];
                let params = if parts.len() > 1 { parts[1].split('&').collect::<Vec<_>>() } else { Vec::new() };
                let params = params.into_iter().map(|x| x.split('=').collect::<Vec<_>>()).filter(|x| x.len() > 1).map(|x| (x[0], x[1])).collect();

                match self.callbacks.get(uri) {
                    Some(callback) => callback(&self, params),
                    None => ("404 Not Found".into(), "Nothing here".into()), 
                }
            },
            _ => ("400 Bad Request".into(), "You are stupid".into()),
        };

        //let contents = fs::read_to_string(filename).unwrap();

        let status_line = format!("HTTP/1.1 {}", status_code);

        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            contents.len(),
            contents
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn query_region_list(&self, args: Vec<(&str, &str)>) -> (String, String) {
        println!("Args: {:?}", args);

        let keys = self.load_keys("master");
        
        let mut region_info = proto::RegionSimpleInfo::default();
        region_info.name = "private_server".into();
        region_info.title = "Private Server".into();
        region_info.r#type = "DEV_PUBLIC".into();
        region_info.dispatch_url = format!("http://localhost:{}/query_cur_region", self.port);

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

        return ("200 OK".into(), base64::encode(region_list_buf));
    }

    fn query_cur_region(&self, args: Vec<(&str, &str)>) -> (String, String) {
        println!("Args: {:?}", args);

        let keys = self.load_keys("master");
        
        let mut region_info = proto::RegionInfo::default();
        region_info.gateserver_ip = "127.0.0.1".to_string();
        region_info.gateserver_port = 4242;
        region_info.secret_key = keys.0.clone();

        let mut region_config = proto::QueryCurrRegionHttpRsp::default();
        region_config.region_info = Some(region_info);
        region_config.client_secret_key = keys.0.clone();

        let json_config = format!("{{\"coverSwitch\": [\"8\"], \"perf_report_config_url\": \"http://localhost:{}/config/verify\", \"perf_report_record_url\": \"http://localhost:{}/dataUpload\" }}",
            self.port, self.port);

        let mut custom_config = json_config.as_bytes().to_owned();

        mhycrypt::mhy_xor(&mut custom_config, &keys.1);

        region_config.region_custom_config_encrypted = custom_config.to_vec();

        let mut region_conf_buf = Vec::new();

        region_config.encode(&mut region_conf_buf).unwrap();

        return ("200 OK".into(), base64::encode(region_conf_buf));
    }

    fn risky_api_check(&self, args: Vec<(&str, &str)>) -> (String, String) {
        let email = "ceo@hoyolab.com";
        let name = "Ceo";
        let token = "Fake-token-hahaha";
        let uid = 0x1234;

        let payload = self.build_account_data(email, name, token, uid);

        return ("200 OK".into(), self.make_answer(0, &payload));
    }

    fn granter_login(&self, args: Vec<(&str, &str)>) -> (String, String) {
        let payload = self.verify_token_v2();

        return ("200 OK".into(), self.make_answer(0, &payload));
    }

    fn shield_verify(&self, args: Vec<(&str, &str)>) -> (String, String) {
        let email = "ceo@hoyolab.com";
        let name = "Ceo";
        let token = "Fake-token-hahaha";
        let uid = 0x1234;

        let payload = self.build_account_data(email, name, token, uid);

        return ("200 OK".into(), self.make_answer(0, &payload));
    }

    fn load_keys(&self, name: &str) -> (Vec<u8>, Vec<u8>) {
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

    fn verify_token_v2(&self) -> String {
        let account_type = 1;
        let combo_id = 0x4321;
        let combo_token = "Fake-token-hehehe";
        let open_id = 0x1234;

        return format!("{{
            \"account_type\": \"{}\",
            \"combo_id\": \"{}\",
            \"combo_token\": \"{}\",
            \"data\": {{\"guest\": \"false\"}},
            \"heartbeat\": \"false\",
            \"open_id\": \"{}\"
        }}", account_type, combo_id, combo_token, open_id);
    }

    fn build_account_data(&self, email: &str, name: &str, token: &str, uid: i32) -> String {
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

    fn make_answer(&self, code: i32, data: &str) -> String {
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
