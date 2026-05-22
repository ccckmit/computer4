use std::cell::RefCell;
use std::fs;

#[derive(Debug, Clone)]
pub enum DeviceType {
    PNG,
    SVG,
}

#[derive(Debug, Clone)]
pub struct DeviceParams {
    pub filename: String,
    pub width: u32,
    pub height: u32,
    pub device_type: DeviceType,
}

impl DeviceParams {
    pub fn new(filename: &str, width: u32, height: u32, device_type: DeviceType) -> Self {
        DeviceParams {
            filename: filename.to_string(),
            width,
            height,
            device_type,
        }
    }
}

thread_local! {
    static CURRENT_DEVICE: RefCell<Option<DeviceParams>> = RefCell::new(None);
}

pub fn png(filename: &str, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
    let params = DeviceParams::new(filename, width, height, DeviceType::PNG);
    CURRENT_DEVICE.with(|cell| {
        *cell.borrow_mut() = Some(params);
    });
    Ok(())
}

pub fn svg(filename: &str, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
    let params = DeviceParams::new(filename, width, height, DeviceType::SVG);
    CURRENT_DEVICE.with(|cell| {
        *cell.borrow_mut() = Some(params);
    });
    Ok(())
}

pub fn dev_off() -> Result<(), Box<dyn std::error::Error>> {
    CURRENT_DEVICE.with(|cell| {
        *cell.borrow_mut() = None;
    });
    Ok(())
}

pub fn clear() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn get_device() -> Option<DeviceParams> {
    CURRENT_DEVICE.with(|cell| cell.borrow().clone())
}