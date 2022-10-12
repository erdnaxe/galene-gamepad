use evdev::{uinput::VirtualDevice, AbsoluteAxisType, FFEffectType, Key};
use log::info;

const XBOX_KEYS: [evdev::Key; 11] = [
    Key::BTN_SOUTH, // A
    Key::BTN_EAST,  // B
    Key::BTN_NORTH, // X
    Key::BTN_WEST,  // Y
    Key::BTN_TL,
    Key::BTN_TR,
    Key::BTN_SELECT,
    Key::BTN_START,
    Key::BTN_MODE, // Home
    Key::BTN_THUMBL,
    Key::BTN_THUMBR,
];

const XBOX_FF: [FFEffectType; 6] = [
    FFEffectType::FF_RUMBLE,
    FFEffectType::FF_PERIODIC,
    FFEffectType::FF_SQUARE,
    FFEffectType::FF_TRIANGLE,
    FFEffectType::FF_SINE,
    FFEffectType::FF_GAIN,
];

/// Setup virtual gamepad device to behave like a Xbox 360 controller
pub fn setup() -> std::io::Result<VirtualDevice> {
    let keys = evdev::AttributeSet::from_iter(XBOX_KEYS);

    let abs_setup_stick = evdev::AbsInfo::new(0, -32768, 32767, 16, 128, 0);
    let abs_x = evdev::UinputAbsSetup::new(AbsoluteAxisType::ABS_X, abs_setup_stick);
    let abs_y = evdev::UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, abs_setup_stick);
    let abs_rx = evdev::UinputAbsSetup::new(AbsoluteAxisType::ABS_RX, abs_setup_stick);
    let abs_ry = evdev::UinputAbsSetup::new(AbsoluteAxisType::ABS_RY, abs_setup_stick);

    let abs_setup_trigger = evdev::AbsInfo::new(0, 0, 255, 0, 0, 0);
    let abs_z = evdev::UinputAbsSetup::new(AbsoluteAxisType::ABS_Z, abs_setup_trigger);
    let abs_rz = evdev::UinputAbsSetup::new(AbsoluteAxisType::ABS_RZ, abs_setup_trigger);

    let abs_setup_dpad = evdev::AbsInfo::new(0, -1, 1, 0, 0, 0);
    let abs_hat0x = evdev::UinputAbsSetup::new(AbsoluteAxisType::ABS_HAT0X, abs_setup_dpad);
    let abs_hat0y = evdev::UinputAbsSetup::new(AbsoluteAxisType::ABS_HAT0Y, abs_setup_dpad);

    let ff = evdev::AttributeSet::from_iter(XBOX_FF);

    let controller_id = evdev::InputId::new(evdev::BusType::BUS_USB, 0x45e, 0x28e, 0x114);
    let mut device = evdev::uinput::VirtualDeviceBuilder::new()?
        .input_id(controller_id)
        .name("galene-gamepad virtual controller")
        .with_keys(&keys)?
        .with_absolute_axis(&abs_x)?
        .with_absolute_axis(&abs_y)?
        .with_absolute_axis(&abs_rx)?
        .with_absolute_axis(&abs_ry)?
        .with_absolute_axis(&abs_z)?
        .with_absolute_axis(&abs_rz)?
        .with_absolute_axis(&abs_hat0x)?
        .with_absolute_axis(&abs_hat0y)?
        .with_ff(&ff)?
        .with_ff_effects_max(16)
        .build()?;

    for path in device.enumerate_dev_nodes_blocking()? {
        let path = path?;
        info!("Virtual controller available as {path:?}");
    }

    Ok(device)
}

pub fn press_release(
    device: &mut VirtualDevice,
    type_: evdev::EventType,
    code: u16,
    value: i32,
    delay: u64,
) -> std::io::Result<()> {
    // Press
    let down_event = evdev::InputEvent::new(type_, code, value);
    device.emit(&[down_event])?;

    std::thread::sleep(std::time::Duration::from_millis(delay));

    // Release
    let up_event = evdev::InputEvent::new(type_, code, 0);
    device.emit(&[up_event])?;

    Ok(())
}
