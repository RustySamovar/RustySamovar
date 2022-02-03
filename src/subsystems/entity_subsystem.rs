use std::sync::{mpsc, Arc};
use std::collections::HashMap;

use crate::server::IpcMessage;

use prost::Message;

use proto;
use proto::{PacketId, CombatTypeArgument, ForwardType};

use packet_processor_macro::*;
#[macro_use]
use packet_processor::*;

#[packet_processor(
CombatInvocationsNotify,
)]
pub struct EntitySubsystem {
    packets_to_send_tx: mpsc::Sender<IpcMessage>,
}

impl EntitySubsystem {
    pub fn new(packets_to_send_tx: mpsc::Sender<IpcMessage>) -> EntitySubsystem {
        let mut es = EntitySubsystem {
            packets_to_send_tx: packets_to_send_tx,
            packet_callbacks: HashMap::new(),
        };

        es.register();

        return es;
    }

    fn process_combat_invocations(&self, user_id: u32, metadata: &proto::PacketHead, notify: &proto::CombatInvocationsNotify) {
        for invoke in notify.invoke_list.iter() {
            self.handle_invoke(user_id, metadata, invoke);
            self.forward_invoke(user_id, metadata, invoke);
        }
    }

    /*
        Invocation handlers
     */
    fn handle_invoke(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        match CombatTypeArgument::from_i32(invoke.argument_type).unwrap() { // Panics in case of unknown (undescribed in protobuf file) argument type
            CombatTypeArgument::CombatNone                       => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatEvtBeingHit                => self.ih_being_hit(user_id, metadata, &EasilyUnpackable::from(&invoke.combat_data)),
            CombatTypeArgument::CombatAnimatorStateChanged       => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatFaceToDir                  => self.ih_face_to_dir(user_id, metadata, &EasilyUnpackable::from(&invoke.combat_data)),
            CombatTypeArgument::CombatSetAttackTarget            => self.ih_set_attack_target(user_id, metadata, &EasilyUnpackable::from(&invoke.combat_data)),
            CombatTypeArgument::CombatRushMove                   => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatAnimatorParameterChanged   => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::EntityMove                       => self.ih_entity_move(user_id, metadata, &EasilyUnpackable::from(&invoke.combat_data)),
            CombatTypeArgument::SyncEntityPosition               => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatSteerMotionInfo            => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatForceSetPosInfo            => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatCompensatePosDiff          => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatMonsterDoBlink             => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatFixedRushMove              => self.ih_default(user_id, metadata, invoke),
            CombatTypeArgument::CombatSyncTransform              => self.ih_default(user_id, metadata, invoke),
            _ => panic!("Unhandled CombatTypeArgument {}", invoke.argument_type), // Panics in case of unknown (unhandled) argument type
        }
    }

    fn ih_default(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        // TODO: this handler is just a stub!
        println!("Unhandled CIN invoke: {:?}", invoke);
    }

    fn ih_entity_move(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::EntityMoveInfo) {

    }

    fn ih_set_attack_target(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::EvtSetAttackTargetInfo) {

    }

    fn ih_face_to_dir(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::EvtFaceToDirInfo) {

    }

    fn ih_being_hit(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::EvtBeingHitInfo) {

    }

    /*
        Forward handlers
     */

    // Main function
    fn forward_invoke(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        match ForwardType::from_i32(invoke.forward_type).unwrap() { // Panics in case of unknown (undescribed in protobuf file) forward type
            ForwardType::ForwardLocal                 => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToAll                 => self.fw_to_all(user_id, metadata, invoke),
            ForwardType::ForwardToAllExceptCur        => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToHost                => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToAllGuest            => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToPeer                => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToPeers               => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardOnlyServer            => self.fw_default(user_id, metadata, invoke),
            ForwardType::ForwardToAllExistExceptCur   => self.fw_default(user_id, metadata, invoke),
            _ => panic!("Unhandled ForwardType {}", invoke.forward_type), // Panics in case of unknown (unhandled) forward type
        }
    }

    fn fw_default(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        // TODO: this handler is just a stub!
        println!("Unhandled CIN forward: {:?}", invoke);
    }

    fn fw_to_all(&self, user_id: u32, metadata: &proto::PacketHead, invoke: &proto::CombatInvokeEntry) {
        // TODO: this handler sends data only back to the user itself for now!
        build_and_send!(self, user_id, metadata, CombatInvocationsNotify {
            invoke_list: vec![invoke.clone()],
        });
    }
}