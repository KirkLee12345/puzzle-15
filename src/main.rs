#![windows_subsystem = "windows"]

use egui_chinese_font::setup_chinese_fonts;
use eframe::egui::ViewportBuilder;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([600.0, 600.0])   // 初始大小（宽，高）
            .with_resizable(true),             // 允许用户调整大小
        ..Default::default()
    };
    eframe::run_native(
        "华容道-15",
        options,
        Box::new(|cc| {
            setup_chinese_fonts(&cc.egui_ctx).expect("无法加载中文字体");
            egui_extras::install_image_loaders(&cc.egui_ctx);

            // 设置白色背景样式（不变）
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.panel_fill = egui::Color32::WHITE;
            style.visuals.window_fill = egui::Color32::WHITE;
            style.visuals.widgets.noninteractive.bg_fill = egui::Color32::WHITE;
            cc.egui_ctx.set_style(style);

            Box::new(MyApp::new(&cc.egui_ctx))
        }),
    )
}

struct MyApp {
    grid: [[usize; 4]; 4],
    textures: Vec<egui_extras::RetainedImage>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // 强制正方形窗口
        let rect = ctx.screen_rect();
        let size = rect.size();
        if (size.x - size.y).abs() > 5.0 {
            let new_size = size.x.min(size.y);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(new_size, new_size)));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // 顶部按钮和显示胜利区
            ui.horizontal(|ui| {
                if ui.button("重置").clicked() {
                    self.reset();
                }
                if ui.button("洗牌").clicked() {
                    self.shuffle(200);
                }
                if self.is_win() {
                    ui.colored_label(egui::Color32::BLACK, "        🎉🎉🎉🎉🎉 胜利！🎉🎉🎉🎉🎉");
                }
            });
            ui.add_space(0.0);

            // 计算网格大小
            let available = ui.available_size();
            let board_size = available.x.min(available.y);
            let tile_size = ((board_size - 40.0) / 4.0).floor().max(20.0);

            egui::Grid::new("puzzle_grid").show(ui, |ui| {
                for row in 0..4 {
                    for col in 0..4 {
                        let value = self.grid[row][col];
                        if value == 0 {
                            ui.add_sized([tile_size, tile_size], egui::Label::new(""));
                        } else {
                            if let Some(tex) = self.textures.get(value - 1) {
                                let texture_id = tex.texture_id(ctx);
                                let img = egui::Image::new((texture_id, egui::vec2(tile_size, tile_size)));
                                let button = egui::ImageButton::new(img);
                                let response = ui.add(button);
                                if response.hovered() && ctx.input(|i| i.pointer.primary_pressed()) {
                                    self.try_move(row, col);
                                    if self.is_win() {
                                        ctx.request_repaint();
                                    }
                                }
                            } else {
                                ui.label("?");
                            }
                        }
                    }
                    ui.end_row();
                }
            });

        });
    }
}

impl MyApp {
    fn new(ctx: &egui::Context) -> Self {
        let mut textures = Vec::with_capacity(15);
        for i in 1..=15 {
            let path = format!("./assets/{}.png", i);
            let image_bytes = std::fs::read(&path)
                .expect(&format!("无法读取图片文件: {}", path));
            let img = egui_extras::RetainedImage::from_image_bytes(&path, &image_bytes)
                .expect(&format!("无法解码图片: {}", path));
            textures.push(img);
        }

        // 初始化网格
        let mut grid = [[0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                let value = i * 4 + j + 1;
                if value == 16 {
                    grid[i][j] = 0;
                } else {
                    grid[i][j] = value;
                }
            }
        }
        Self { grid, textures }
    }
    /// 找到空格的位置，返回 (行, 列)
    fn find_empty(&self) -> (usize, usize) {
        for i in 0..4 {
            for j in 0..4 {
                if self.grid[i][j] == 0 {
                    return (i, j);
                }
            }
        }
        unreachable!("应该总是有一个空格")
    }

    /// 判断 (row, col) 是否与空格相邻
    fn is_adjacent_to_empty(&self, row: usize, col: usize) -> bool {
        let (er, ec) = self.find_empty();
        (row == er && (col as i32 - ec as i32).abs() == 1) ||
            (col == ec && (row as i32 - er as i32).abs() == 1)
    }

    /// 尝试移动方块，成功返回 true
    fn try_move(&mut self, row: usize, col: usize) -> bool {
        if self.is_adjacent_to_empty(row, col) {
            let (er, ec) = self.find_empty();
            self.grid[er][ec] = self.grid[row][col];
            self.grid[row][col] = 0;
            true
        } else {
            false
        }
    }

    /// 检查是否胜利（顺序为 1..15，最后是 0）
    fn is_win(&self) -> bool {
        for i in 0..4 {
            for j in 0..4 {
                let expected = i * 4 + j + 1;
                if expected == 16 {
                    if self.grid[i][j] != 0 { return false; }
                } else {
                    if self.grid[i][j] != expected { return false; }
                }
            }
        }
        true
    }

    /// 重置游戏到胜利状态（顺序排列）
    fn reset(&mut self) {
        // 重置网格为胜利状态
        for i in 0..4 {
            for j in 0..4 {
                let value = i * 4 + j + 1;
                if value == 16 {
                    self.grid[i][j] = 0;
                } else {
                    self.grid[i][j] = value;
                }
            }
        }
    }

    /// 洗牌（随机打乱棋盘，同时保证有解）
    fn shuffle(&mut self, steps: usize) {
        self.reset();
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        for _ in 0..steps {
            let (er, ec) = self.find_empty();
            let mut neighbors = Vec::new();
            if er > 0 { neighbors.push((er - 1, ec)); }
            if er < 3 { neighbors.push((er + 1, ec)); }
            if ec > 0 { neighbors.push((er, ec - 1)); }
            if ec < 3 { neighbors.push((er, ec + 1)); }
            if let Some(&(nr, nc)) = neighbors.choose(&mut rng) {
                self.try_move(nr, nc);
            }
        }
    }
}