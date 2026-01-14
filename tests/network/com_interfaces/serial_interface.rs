use datex_core::{
    network::com_interfaces::{
        com_interface::ComInterface,
        default_com_interfaces::serial::{
            serial_common::SerialInterfaceSetupData,
        },
    },
    utils::context::init_global_context,
};
use log::info;
use datex_core::runtime::AsyncContext;

#[tokio::test]
pub async fn test_construct() {
    init_global_context();
    const PORT_NAME: &str = "/dev/ttyUSB0";
    const BAUD_RATE: u32 = 115200;
    let available_ports = SerialInterfaceSetupData::get_available_ports();
    for port in available_ports.clone() {
        info!("Available port: {port}");
    }
    if !available_ports.contains(&PORT_NAME.to_string()) {
        return;
    }
    let mut interface = ComInterface::create_sync_from_setup_data(SerialInterfaceSetupData {
        port_name: Some(PORT_NAME.to_string()),
        baud_rate: BAUD_RATE,
    }, AsyncContext::default())
    .unwrap();
}
