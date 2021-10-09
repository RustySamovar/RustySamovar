use std::io::Result;

fn main() -> Result<()> {
    let proto_dir = "protobuf";

    let protos = vec![
        // Dispatch protocol
        "QueryRegionListHttpRsp",
        "QueryCurrRegionHttpRsp",
        "RegionSimpleInfo",
        "RegionInfo",

        // Game protocol
        "packet_header",

        "GetPlayerTokenReq",
        "GetPlayerTokenRsp",
        "PlayerLoginReq",
        "OpenStateUpdateNotify",
        "StoreWeightLimitNotify",
        "PlayerStoreNotify",
        "AvatarDataNotify",
        "PlayerEnterSceneNotify",
        "PlayerLoginRsp",
        "GetPlayerSocialDetailReq",
        "GetPlayerSocialDetailRsp",
        "EnterSceneReadyReq",
        "EnterSceneReadyRsp",
        "SceneInitFinishReq",
        "EnterScenePeerNotify",
        "WorldDataNotify",
        "WorldPlayerInfoNotify",
        "ScenePlayerInfoNotify",
        "PlayerEnterSceneInfoNotify",
        "PlayerGameTimeNotify",
        "SceneTimeNotify",
        "SceneDataNotify",
        "HostPlayerNotify",
        "SceneTeamUpdateNotify",
        "SceneInitFinishRsp",
        "EnterSceneDoneReq",
        "SceneEntityAppearNotify",
        "EnterSceneDoneRsp",
        "PostEnterSceneReq",
        "PostEnterSceneRsp",
        
        "WorldPlayerRTTNotify",
        "PingReq",
        "PingRsp",
        "PlayerDataNotify",

        "EnterWorldAreaReq",
        "EnterWorldAreaRsp",

    ];

    let protos: Vec<String> = protos.iter().map(|&x| format!("{}/{}.proto", proto_dir, x)).collect();

    let ret = prost_build::compile_protos(&protos, &[format!("{}/", proto_dir)]);

    match ret {
        Ok(_) => return Ok(()),
        Err(e) => panic!("{}", e),
    }
}
