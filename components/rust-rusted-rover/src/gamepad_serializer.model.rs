#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GamePadState {
    #[prost(uint32, tag="1")]
    pub gamepad_id: u32,
    #[prost(float, tag="2")]
    pub left_y: f32,
    #[prost(float, tag="3")]
    pub left_x: f32,
    #[prost(float, tag="4")]
    pub right_x: f32,
    #[prost(float, tag="5")]
    pub right_y: f32,
    #[prost(bool, tag="6")]
    pub north: bool,
    #[prost(bool, tag="7")]
    pub south: bool,
    #[prost(bool, tag="8")]
    pub west: bool,
    #[prost(bool, tag="9")]
    pub east: bool,
    #[prost(float, tag="10")]
    pub left_trigger_2: f32,
    #[prost(float, tag="11")]
    pub right_trigger_2: f32,
}
