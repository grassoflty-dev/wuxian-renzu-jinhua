//! Probe-only consumer of the unchanged formal library source.
//! This adapter implements no capability, owner, provider, HUD, or lifecycle shim.

#[cfg(test)]
mod tests {
    use wuxian_horror_ch1::{campaign, continuous_input, world_v3};

    #[test]
    fn public_campaign_api_is_callable_and_serializable() {
        let state = campaign::CampaignState::default();
        assert!(campaign::view(&state).is_object());
        assert!(serde_json::to_value(&state).unwrap().is_object());
    }

    #[test]
    fn public_input_epoch_boundary_is_available() {
        let mut receiver = continuous_input::InputReceiver::new(7);
        receiver.set_world_epoch(8);
        assert_eq!(receiver.pending_len(), 0);
    }

    #[test]
    fn public_world_transform_api_is_available() {
        let position = world_v3::Vec3::new(1.0, 0.0, 2.0).unwrap();
        let transform = world_v3::Transform::new(position, 0.5).unwrap();
        assert!(serde_json::to_value(transform).unwrap().is_object());
    }
}
