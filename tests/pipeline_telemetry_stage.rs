use airborne_cot_relay::pipeline::Pipeline;
use airborne_cot_relay::RelayConfig;
use stm32_navcore::protocol::NavTelemetry;

fn frame(timestamp_ms: u32) -> Vec<u8> {
    let telem = NavTelemetry {
        timestamp_ms,
        pos_ned_m: [0.0, 0.0, -100.0],
        vel_ned_mps: [0.0; 3],
        euler_rad: [0.0; 3],
        status: 0,
    };
    let mut buf = [0u8; 64];
    let len = telem.pack_uart(&mut buf).unwrap();
    buf[..len].to_vec()
}

fn pipeline() -> Pipeline {
    Pipeline::from_config(&RelayConfig::default()).unwrap()
}
