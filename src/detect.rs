use image::GrayImage;
use std::{thread, time::Duration};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::Mutex;
use xcap::Monitor;
use enigo::{Enigo, Key, Keyboard, Settings};

use crate::config::AppConfig;

pub fn start_capture_thread(
    active: Arc<AtomicBool>,
    config: Arc<Mutex<AppConfig>>,
) {
    thread::spawn(move || {
        let mut enigo = Enigo::new(&Settings::default()).expect("Failed to init enigo");
        
        loop {
            if !active.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            // Get current config
            let (x, y, w, h) = {
                let cfg = config.lock();
                (cfg.ocr.capture_x, cfg.ocr.capture_y, cfg.ocr.capture_width, cfg.ocr.capture_height)
            };

            // Get primary monitor (usually 0) - this is a simplification.
            // Python code uses 'dxcam' which defaults to primary.
            let monitors = Monitor::all().unwrap_or_default();
            if monitors.is_empty() {
                thread::sleep(Duration::from_secs(1));
                continue;
            }
            let monitor = &monitors[0]; // Assuming primary is first or user wants first

            // Convert config coords (which might be screen relative or window relative) 
            // to monitor relative if needed?
            // Python code uses 'winfo_screenwidth' logic, implying absolute screen coords.
            // xcap takes x,y relative to the monitor top-left normally.
            
            // Normalize coords
            let mx = if x < 0 { 0 } else { x };
            let my = if y < 0 { 0 } else { y };
            
            // Capture
            // xcap capture_area takes (x, y, w, h)
            // Capture
            // Use capture_image() which is more standard in xcap 0.1
            if let Ok(image) = monitor.capture_image() {
                // Crop to region
                // image is RgbaImage
                // Validate bounds
                let img_w = image.width();
                let img_h = image.height();
                
                let cx = mx.min(img_w as i32) as u32;
                let cy = my.min(img_h as i32) as u32;
                let cw = w.min(img_w - cx);
                let ch = h.min(img_h - cy);
                
                if cw == 0 || ch == 0 {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }

                // Easiest is to just iterate the relevant pixels directly instead of creating a new image
                // or use generic functionality
                
                // Let's create a cropped GrayImage directly
                let mut gray = GrayImage::new(cw, ch);
                for y in 0..ch {
                    for x in 0..cw {
                        let pixel = image.get_pixel(cx + x, cy + y);
                        // Convert to luma
                        let luma = pixel.0[0] as f32 * 0.299 + pixel.0[1] as f32 * 0.587 + pixel.0[2] as f32 * 0.114;
                        gray.put_pixel(x, y, image::Luma([luma as u8]));
                    }
                }
                
                // Threshold & detect
                if detect_white_blob(&gray) {
                    let _ = enigo.key(Key::Return, enigo::Direction::Click);
                    thread::sleep(Duration::from_millis(500)); 
                }
            }

            // sleep a bit to save CPU - Python doesn't sleep explicitly but `grab` blocks/waits for frame? 
            // dxcam is fast. Let's sleep 16ms (~60fps)
            thread::sleep(Duration::from_millis(50));
        }
    });
}



fn detect_white_blob(gray: &GrayImage) -> bool {
    // Threshold 240
    // Check for contours > 40x40. 
    // Simplified: check for a connected component or just any blob of pixels that fits the criteria?
    // Python: cv2.threshold(gray, 240, 255, cv2.THRESH_BINARY)
    // cv2.findContours... if w > 40 and h > 40
    
    // We can do a simpler check:
    // Iterate pixels, find bounding box of all pixels > 240.
    // If bounding box width > 40 and height > 40, return true.
    // This is an approximation but likely sufficient for "white box on dark background".
    
    let mut min_x = gray.width();
    let mut max_x = 0;
    let mut min_y = gray.height();
    let mut max_y = 0;
    let mut found = false;

    for y in 0..gray.height() {
        for x in 0..gray.width() {
            if gray.get_pixel(x, y)[0] >= 240 {
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
                found = true;
            }
        }
    }

    if !found {
        return false;
    }

    let w = if max_x >= min_x { max_x - min_x } else { 0 };
    let h = if max_y >= min_y { max_y - min_y } else { 0 };

    w > 40 && h > 40
}
