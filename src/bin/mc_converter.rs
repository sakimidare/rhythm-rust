use mug_tui::convert::mc::convert_mc_to_custom;
use std::fs;

fn main() -> anyhow::Result<()> {
    // 1. 配置输入输出路径
    let input_path = "chart.mc"; // 你的 Malody 源文件
    let output_dir = "output_song";   // 输出目录

    // 创建输出目录结构
    fs::create_dir_all(format!("{}/charts", output_dir))?;

    // 2. 读取并转换
    println!("Reading Malody chart: {}...", input_path);
    let mc_content = fs::read_to_string(input_path)?;

    // 调用你之前实现的转换函数
    let (chart, song_meta) = convert_mc_to_custom(&mc_content);

    // 3. 输出谱面 JSON (charts/at.json)
    let chart_json = serde_json::to_string_pretty(&chart)?;
    let chart_output = format!("{}/charts/at.json", output_dir);
    fs::write(&chart_output, chart_json)?;
    println!("Generated chart: {}", chart_output);

    // 4. 构建并输出 song.json
    // 注意：这里需要补全你 song.json 结构中缺失的路径信息
    let full_song_config = serde_json::json!({
        "meta": song_meta,
        "audio_file": "audio.mp3",          // 默认占位符，需手动改
        "chart_files": ["charts/at.json"], // 对应刚才生成的谱面
        "illu_file": "bg.png"               // 默认占位符
    });

    let song_json = serde_json::to_string_pretty(&full_song_config)?;
    let song_output = format!("{}/song.json", output_dir);
    fs::write(&song_output, song_json)?;
    println!("Generated config: {}", song_output);

    println!("\nConversion complete! Folder '{}' is ready for testing.", output_dir);
    Ok(())
}