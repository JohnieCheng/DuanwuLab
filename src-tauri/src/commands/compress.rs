use image::ImageEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
pub use shared_types::image_compressor::{CompressResult, PickedFile};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn pick_image_files(app: tauri::AppHandle) -> Vec<PickedFile> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().add_filter("Images", &["jpg", "jpeg", "png", "webp"]).pick_files(
        move |files| {
            let _ = tx.send(files);
        },
    );

    rx.recv()
        .unwrap_or_default()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| {
            let path = p.as_path()?.to_string_lossy().to_string();
            let name = p.as_path()?.file_name()?.to_string_lossy().to_string();
            let size = std::fs::metadata(p.as_path()?).map(|m| m.len()).unwrap_or(0);
            Some(PickedFile { path, size, name })
        })
        .collect()
}

#[tauri::command]
pub async fn pick_output_dir(app: tauri::AppHandle) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |folder| {
        let _ = tx.send(folder);
    });

    if let Ok(Some(folder_path)) = rx.recv() {
        folder_path.as_path().map(|p| p.to_string_lossy().to_string())
    } else {
        None
    }
}

#[tauri::command]
pub async fn compress_single_image(
    path: String,
    quality: u8,
    mode: String,
    output_dir: Option<String>, // 前端必须传 "outputDir"
) -> Result<CompressResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        match compress_single(&path, quality, &mode, output_dir) {
            Ok(res) => res,
            Err(err_msg) => {
                let p = std::path::Path::new(&path);
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
                let original = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
                CompressResult {
                    original_path: path,
                    compressed_path: String::new(),
                    name,
                    original,
                    compressed: 0,
                    width: 0,
                    height: 0,
                    format: String::new(),
                    error: Some(err_msg),
                    quality,
                }
            }
        }
    })
    .await
    .map_err(|e| e.to_string())
}

/// PNG 有损量化逻辑
fn compress_png(img: &image::DynamicImage, quality: u8, mode: &str) -> Result<Vec<u8>, String> {
    let rgba_img = img.to_rgba8();
    let width = rgba_img.width() as usize;
    let height = rgba_img.height() as usize;

    if quality < 100 {
        let mut liq = imagequant::Attributes::new();
        let min_q = (quality / 2).max(1);
        let _ = liq.set_quality(min_q, quality);

        let input_pixels: &[imagequant::RGBA] = unsafe {
            std::slice::from_raw_parts(rgba_img.as_ptr() as *const imagequant::RGBA, width * height)
        };

        if let Ok(mut liq_img) = liq.new_image(input_pixels, width, height, 0.0) {
            if let Ok(mut res) = liq.quantize(&mut liq_img) {
                let _ = res.set_dithering_level(0.8);

                if let Ok((palette, remapped_pixels)) = res.remapped(&mut liq_img) {
                    let mut out_buf = Vec::new();
                    let mut raw_palette = Vec::with_capacity(palette.len() * 3);
                    for c in &palette {
                        raw_palette.push(c.r);
                        raw_palette.push(c.g);
                        raw_palette.push(c.b);
                    }

                    let encode_res: Result<(), String> = {
                        let mut png_writer =
                            png::Encoder::new(&mut out_buf, width as u32, height as u32);
                        png_writer.set_color(png::ColorType::Indexed);
                        png_writer.set_depth(png::BitDepth::Eight);
                        png_writer.set_palette(raw_palette);

                        let trns: Vec<u8> = palette.iter().map(|c| c.a).collect();
                        if trns.iter().any(|&a| a < 255) {
                            png_writer.set_trns(trns);
                        }

                        let mut writer = png_writer
                            .write_header()
                            .map_err(|e| format!("PNG header error: {e}"))?;
                        writer
                            .write_image_data(&remapped_pixels)
                            .map_err(|e| format!("PNG write error: {e}"))?;
                        writer.finish().map_err(|e| format!("PNG finish error: {e}"))?;
                        Ok(())
                    };

                    if encode_res.is_ok() {
                        return Ok(out_buf);
                    }
                }
            }
        }
    }

    // 回退到无损 RGBA PNG 编码
    let mut out_buf = Vec::new();
    let (png_comp, png_filter) = if mode == "best" {
        (CompressionType::Best, FilterType::Adaptive)
    } else {
        (CompressionType::Fast, FilterType::Sub)
    };

    let encoder = PngEncoder::new_with_quality(&mut out_buf, png_comp, png_filter);
    encoder
        .write_image(
            rgba_img.as_raw(),
            width as u32,
            height as u32,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("PNG encode error: {e}"))?;

    Ok(out_buf)
}

fn compress_single(
    path_str: &str,
    quality: u8,
    mode: &str,
    output_dir: Option<String>,
) -> Result<CompressResult, String> {
    let path = std::path::Path::new(path_str);

    let raw_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
    // 清理掉多次压缩可能产生的后缀
    let clean_stem = raw_stem.replace(".compressed", "").replace("-compressed", "");

    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
    let original = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let mut img = image::open(path).map_err(|e| format!("Open error: {e}"))?;

    let max_dim = 3840;
    if img.width() > max_dim || img.height() > max_dim {
        img = img.resize(max_dim, max_dim, image::imageops::FilterType::Triangle);
    }

    let (w, h) = (img.width(), img.height());
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("jpg").to_lowercase();

    let out_ext = if ext == "png" {
        "png"
    } else if ext == "webp" {
        "webp"
    } else {
        "jpg"
    };
    let out_name = format!("{}.compressed.{}", clean_stem, out_ext);

    // 核心落地路径生成逻辑
    let out_path = if let Some(dir) = output_dir {
        std::path::Path::new(&dir).join(&out_name)
    } else {
        path.with_file_name(&out_name)
    };

    let mut out_buf = Vec::with_capacity(original as usize);

    let data = match out_ext {
        "jpg" | "jpeg" => {
            let mut encoder = JpegEncoder::new_with_quality(&mut out_buf, quality);
            encoder.encode_image(&img).map_err(|e| format!("JPEG encode error: {e}"))?;
            out_buf
        }
        "png" => compress_png(&img, quality, mode)?,
        "webp" => {
            img.write_to(&mut std::io::Cursor::new(&mut out_buf), image::ImageFormat::WebP)
                .map_err(|e| format!("WebP encode error: {e}"))?;
            out_buf
        }
        _ => return Err("Unsupported format".to_string()),
    };

    let compressed_size = data.len() as u64;

    // 写入或覆盖原文件保护
    let (final_path, final_size) = if compressed_size >= original && original > 0 {
        std::fs::copy(path, &out_path).map_err(|e| format!("Copy original file error: {e}"))?;
        (out_path.to_string_lossy().to_string(), original)
    } else {
        std::fs::write(&out_path, &data).map_err(|e| format!("Save error: {e}"))?;
        (out_path.to_string_lossy().to_string(), compressed_size)
    };

    Ok(CompressResult {
        original_path: path_str.to_string(),
        compressed_path: final_path,
        name,
        original,
        compressed: final_size,
        width: w,
        height: h,
        format: out_ext.to_string(),
        error: None,
        quality,
    })
}

#[tauri::command]
pub async fn preview_compress(path: String, quality: u8) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let p = std::path::Path::new(&path);
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("jpg").to_lowercase();
        let original_size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        let mut img = image::open(p).map_err(|e| format!("Open error: {e}"))?;

        let max_dim = 2560;
        if img.width() > max_dim || img.height() > max_dim {
            img = img.resize(max_dim, max_dim, image::imageops::FilterType::Triangle);
        }

        let (w, h) = (img.width(), img.height());

        let out_buf = if ext == "png" {
            compress_png(&img, quality, "fast")?
        } else {
            let mut buf = Vec::new();
            let mut encoder = JpegEncoder::new_with_quality(&mut buf, quality);
            encoder.encode_image(&img).map_err(|e| format!("JPEG encode error: {e}"))?;
            buf
        };

        let compressed_size = out_buf.len() as u64;
        let tmp_comp =
            std::env::temp_dir().join(format!("dc_comp_{}.{}", uuid::Uuid::new_v4(), ext));
        std::fs::write(&tmp_comp, &out_buf).map_err(|e| format!("Failed to save preview: {e}"))?;

        Ok(serde_json::json!({
            "original": path,
            "compressed": tmp_comp.to_string_lossy().to_string(),
            "original_size": original_size,
            "compressed_size": compressed_size,
            "width": w,
            "height": h,
        }))
    })
    .await
    .map_err(|e| e.to_string())?
}
