#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod backup;

const APP_VERSION: &str = "1.2.0";
use eframe::egui;
use egui::{Color32, RichText};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const DMC_CSV: &str = include_str!("dmc_colors.csv");

#[derive(Clone, Debug)]
struct DmcColor {
    code: String,
    name: String,
    hex: String,
    rgb: [u8; 3],
}

#[derive(Clone, Debug, Default)]
struct InventoryEntry {
    bobbins: u32,
    skeins: u32,
    notes: String,
}

impl InventoryEntry {
    fn total_units(&self) -> u32 {
        self.bobbins + self.skeins
    }

    fn is_empty(&self) -> bool {
        self.bobbins == 0 && self.skeins == 0 && self.notes.trim().is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FilterMode {
    All,
    Owned,
    Missing,
    BobbinsOnly,
    SkeinsOnly,
    LowStock,
    WithNotes,
}

impl FilterMode {
    fn label(self) -> &'static str {
        match self {
            FilterMode::All => "All",
            FilterMode::Owned => "Owned only",
            FilterMode::Missing => "Missing only",
            FilterMode::BobbinsOnly => "Bobbins only",
            FilterMode::SkeinsOnly => "Skeins only",
            FilterMode::LowStock => "Low stock",
            FilterMode::WithNotes => "With notes",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppTab {
    Collection,
    BackupRestore,
    MissingColors,
    PatternPlanner,
    About,
}

struct FlossKeeperApp {
    colors: Vec<DmcColor>,
    inventory: BTreeMap<String, InventoryEntry>,
    search: String,
    selected_code: Option<String>,
    filter_mode: FilterMode,
    status: String,
    save_path: PathBuf,
    dirty: bool,
    active_tab: AppTab,
    backup_status: String,
    missing_status: String,
    pattern_input: String,
    pattern_status: String,
    show_about_window: bool,
    show_first_run_help: bool,
    hide_first_run_help: bool,
    first_run_help_path: PathBuf,
}

impl Default for FlossKeeperApp {
    fn default() -> Self {
        let colors = load_dmc_colors();
        let save_path = default_save_path();
        let first_run_help_path = default_first_run_help_path();
        let first_run_help_dismissed = first_run_help_path.exists();
        let mut app = Self {
            colors,
            inventory: BTreeMap::new(),
            search: String::new(),
            selected_code: None,
            filter_mode: FilterMode::All,
            status: String::new(),
            save_path,
            dirty: false,
            active_tab: AppTab::Collection,
            backup_status: String::new(),
            missing_status: String::new(),
            pattern_input: String::new(),
            pattern_status: String::new(),
            show_about_window: false,
            show_first_run_help: !first_run_help_dismissed,
            hide_first_run_help: false,
            first_run_help_path,
        };

        match app.load_collection() {
            Ok(loaded) => {
                if loaded {
                    app.status = "Loaded saved collection.".to_string();
                } else {
                    app.status =
                        "No saved collection yet. Add bobbins/skeins, then click Save.".to_string();
                }
            }
            Err(err) => {
                app.status = format!("Could not load collection: {err}");
            }
        }

        app
    }
}

impl FlossKeeperApp {
    fn show_about_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_about_window;

        egui::Window::new("About FlossKeeper")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading(format!("FlossKeeper v{}", APP_VERSION));
                ui.label("A simple desktop floss stash tracker for cross-stitchers.");
                ui.add_space(8.0);

                ui.label("What it does:");
                ui.label("• Tracks DMC floss colors");
                ui.label("• Tracks bobbins and skeins separately");
                ui.label("• Helps find missing colors");
                ui.label("• Plans pattern shopping lists against your stash");
                ui.label("• Supports backup, restore, and TSV export");

                ui.add_space(8.0);
                ui.label("Your collection is stored locally on this computer.");
                ui.label("No account, cloud sync, or internet connection is required.");

                ui.add_space(10.0);
                if ui.button("Close").clicked() {
                    self.show_about_window = false;
                }
            });

        self.show_about_window = open && self.show_about_window;
    }

    fn show_first_run_help_window(&mut self, ctx: &egui::Context) {
        if !self.show_first_run_help {
            return;
        }

        let mut open = self.show_first_run_help;
        let mut close_clicked = false;

        egui::Window::new("Welcome to FlossKeeper")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.heading(format!("Welcome to FlossKeeper v{}", APP_VERSION));
                ui.label("FlossKeeper helps you keep track of your cross-stitch floss stash.");
                ui.add_space(8.0);

                ui.label("Quick start:");
                ui.label("• Use Collection to add bobbins and skeins you own.");
                ui.label("• Use Missing Colors to see what DMC colors you still need.");
                ui.label("• Use Pattern Planner to paste a pattern color list and compare it against your stash.");
                ui.label("• Use Backup / Restore before making big changes or moving computers.");

                ui.add_space(8.0);
                ui.checkbox(&mut self.hide_first_run_help, "Don't show this again");

                ui.add_space(10.0);
                if ui.button("Got it").clicked() {
                    close_clicked = true;
                }
            });

        if close_clicked {
            open = false;
        }

        if !open && self.hide_first_run_help {
            if let Some(parent) = self.first_run_help_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&self.first_run_help_path, "dismissed\n");
        }

        self.show_first_run_help = open;
    }

    fn selected_color(&self) -> Option<&DmcColor> {
        self.selected_code
            .as_ref()
            .and_then(|code| self.colors.iter().find(|color| &color.code == code))
    }

    fn entry_for(&self, code: &str) -> InventoryEntry {
        let key = canonical_code(code);
        self.inventory.get(&key).cloned().unwrap_or_default()
    }

    fn entry_for_mut(&mut self, code: &str) -> &mut InventoryEntry {
        let key = canonical_code(code);
        self.inventory.entry(key).or_default()
    }

    fn cleanup_entry_if_empty(&mut self, code: &str) {
        let key = canonical_code(code);
        if self
            .inventory
            .get(&key)
            .map(|entry| entry.is_empty())
            .unwrap_or(false)
        {
            self.inventory.remove(&key);
        }
    }

    fn adjust_bobbins(&mut self, code: &str, delta: i32) {
        let entry = self.entry_for_mut(code);
        entry.bobbins = adjust_u32(entry.bobbins, delta);
        self.cleanup_entry_if_empty(code);
        self.dirty = true;
    }

    fn adjust_skeins(&mut self, code: &str, delta: i32) {
        let entry = self.entry_for_mut(code);
        entry.skeins = adjust_u32(entry.skeins, delta);
        self.cleanup_entry_if_empty(code);
        self.dirty = true;
    }

    fn owned_color_count(&self) -> usize {
        self.colors
            .iter()
            .filter(|color| is_owned(&self.entry_for(&color.code)))
            .count()
    }

    fn missing_color_count(&self) -> usize {
        self.colors.len().saturating_sub(self.owned_color_count())
    }

    fn bobbins_only_count(&self) -> usize {
        self.colors
            .iter()
            .filter(|color| {
                let entry = self.entry_for(&color.code);
                entry.bobbins > 0 && entry.skeins == 0
            })
            .count()
    }

    fn skeins_only_count(&self) -> usize {
        self.colors
            .iter()
            .filter(|color| {
                let entry = self.entry_for(&color.code);
                entry.skeins > 0 && entry.bobbins == 0
            })
            .count()
    }

    fn both_count(&self) -> usize {
        self.colors
            .iter()
            .filter(|color| {
                let entry = self.entry_for(&color.code);
                entry.bobbins > 0 && entry.skeins > 0
            })
            .count()
    }

    fn low_stock_count(&self) -> usize {
        self.colors
            .iter()
            .filter(|color| {
                let entry = self.entry_for(&color.code);
                is_owned(&entry) && entry.total_units() <= 1
            })
            .count()
    }

    fn notes_count(&self) -> usize {
        self.colors
            .iter()
            .filter(|color| !self.entry_for(&color.code).notes.trim().is_empty())
            .count()
    }

    fn total_bobbins(&self) -> u32 {
        self.inventory.values().map(|entry| entry.bobbins).sum()
    }

    fn total_skeins(&self) -> u32 {
        self.inventory.values().map(|entry| entry.skeins).sum()
    }

    fn total_units(&self) -> u32 {
        self.total_bobbins() + self.total_skeins()
    }

    fn save_collection(&mut self) -> io::Result<()> {
        if let Some(parent) = self.save_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::File::create(&self.save_path)?;
        writeln!(file, "# FlossKeeper collection v1")?;
        writeln!(file, "# code\tbobbins\tskeins\tnotes")?;

        for color in &self.colors {
            if let Some(entry) = self.inventory.get(&color.code) {
                if !entry.is_empty() {
                    let notes = entry.notes.replace('\t', " ").replace('\n', " ");
                    writeln!(
                        file,
                        "{}\t{}\t{}\t{}",
                        color.code, entry.bobbins, entry.skeins, notes
                    )?;
                }
            }
        }

        self.dirty = false;
        self.status = format!("Saved collection to {}", self.save_path.display());
        Ok(())
    }

    fn load_collection(&mut self) -> io::Result<bool> {
        if !self.save_path.exists() {
            return Ok(false);
        }

        let contents = fs::read_to_string(&self.save_path)?;
        self.inventory.clear();

        for line in contents.lines() {
            let line = line.trim_end();
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let mut parts = line.splitn(4, '\t');
            let code = parts.next().unwrap_or("").trim();
            let bobbins = parts
                .next()
                .unwrap_or("0")
                .trim()
                .parse::<u32>()
                .unwrap_or(0);
            let skeins = parts
                .next()
                .unwrap_or("0")
                .trim()
                .parse::<u32>()
                .unwrap_or(0);
            let notes = parts.next().unwrap_or("").trim().to_string();

            if code.is_empty() {
                continue;
            }

            // DMC White and Blanc are the same thread colour.
            // Older FlossKeeper files may have saved either code, so merge them safely.
            let canonical = canonical_code(code);
            let entry = self.inventory.entry(canonical).or_default();
            entry.bobbins = entry.bobbins.saturating_add(bobbins);
            entry.skeins = entry.skeins.saturating_add(skeins);

            if !notes.trim().is_empty() {
                if entry.notes.trim().is_empty() {
                    entry.notes = notes;
                } else if !entry.notes.contains(notes.trim()) {
                    entry.notes = format!("{} | {}", entry.notes.trim(), notes.trim());
                }
            }
        }

        self.dirty = false;
        Ok(true)
    }

    fn export_dir(&self) -> PathBuf {
        self.save_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn export_missing_report(&mut self) -> io::Result<()> {
        let dir = self.export_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join("flosskeeper_missing_colors.tsv");
        let mut file = fs::File::create(&path)?;

        writeln!(file, "# FlossKeeper missing colours report")?;
        writeln!(file, "# Generated from your current collection")?;
        writeln!(file, "DMC\tName\tHex")?;

        let mut count = 0usize;
        for color in &self.colors {
            let entry = self.entry_for(&color.code);
            if !is_owned(&entry) {
                writeln!(file, "{}\t{}\t{}", color.code, color.name, color.hex)?;
                count += 1;
            }
        }

        self.status = format!("Exported {count} missing colours to {}", path.display());
        Ok(())
    }

    fn export_shopping_list(&mut self) -> io::Result<()> {
        let dir = self.export_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join("flosskeeper_shopping_list.txt");
        let mut file = fs::File::create(&path)?;

        writeln!(file, "FlossKeeper Shopping List")?;
        writeln!(file, "========================")?;
        writeln!(file)?;

        let mut count = 0usize;
        for color in &self.colors {
            let entry = self.entry_for(&color.code);
            if !is_owned(&entry) {
                writeln!(file, "DMC {:>5}  - {}", color.code, color.name)?;
                count += 1;
            }
        }

        writeln!(file)?;
        writeln!(file, "Total missing colours: {count}")?;

        self.status = format!(
            "Exported shopping list with {count} colours to {}",
            path.display()
        );
        Ok(())
    }

    fn export_inventory_csv(&mut self) -> io::Result<()> {
        let dir = self.export_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join("flosskeeper_inventory_export.csv");
        let mut file = fs::File::create(&path)?;

        writeln!(file, "DMC,Name,Bobbins,Skeins,Total,Notes,Hex")?;

        let mut count = 0usize;
        for color in &self.colors {
            let entry = self.entry_for(&color.code);
            if !entry.is_empty() {
                writeln!(
                    file,
                    "{},{},{},{},{},{},{}",
                    csv_escape(&color.code),
                    csv_escape(&color.name),
                    entry.bobbins,
                    entry.skeins,
                    entry.total_units(),
                    csv_escape(&entry.notes),
                    csv_escape(&color.hex),
                )?;
                count += 1;
            }
        }

        self.status = format!("Exported {count} owned colours to {}", path.display());
        Ok(())
    }

    fn ui_top(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("FlossKeeper");
            ui.label("DMC floss collection tracker");
        });

        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("Owned: {}", self.owned_color_count())).strong());
            ui.label(format!("Missing: {}", self.missing_color_count()));
            ui.label(format!("Bobbins: {}", self.total_bobbins()));
            ui.label(format!("Skeins: {}", self.total_skeins()));
            ui.label(format!("Total units: {}", self.total_units()));
            if self.dirty {
                ui.label(RichText::new("Unsaved changes").strong());
            }
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Bobbins only: {}", self.bobbins_only_count()));
            ui.label(format!("Skeins only: {}", self.skeins_only_count()));
            ui.label(format!("Both: {}", self.both_count()));
            ui.label(format!("Low stock: {}", self.low_stock_count()));
            ui.label(format!("With notes: {}", self.notes_count()));
        });

        ui.horizontal(|ui| {
            if ui.button("Save collection").clicked() {
                if let Err(err) = self.save_collection() {
                    self.status = format!("Save failed: {err}");
                }
            }

            if ui.button("Reload from file").clicked() {
                match self.load_collection() {
                    Ok(true) => self.status = "Reloaded collection from file.".to_string(),
                    Ok(false) => self.status = "No collection file found yet.".to_string(),
                    Err(err) => self.status = format!("Reload failed: {err}"),
                }
            }

            if ui.button("Clear search").clicked() {
                self.search.clear();
            }
        });

        ui.horizontal_wrapped(|ui| {
            if ui.button("Export shopping list").clicked() {
                if let Err(err) = self.export_shopping_list() {
                    self.status = format!("Shopping list export failed: {err}");
                }
            }

            if ui.button("Export missing report").clicked() {
                if let Err(err) = self.export_missing_report() {
                    self.status = format!("Missing report export failed: {err}");
                }
            }

            if ui.button("Export owned CSV").clicked() {
                if let Err(err) = self.export_inventory_csv() {
                    self.status = format!("Inventory CSV export failed: {err}");
                }
            }
        });

        ui.label(format!("Save file: {}", self.save_path.display()));
        ui.label(&self.status);
    }

    fn ui_filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Search DMC/name/notes:");
            ui.text_edit_singleline(&mut self.search);
            if ui.button("Clear").clicked() {
                self.search.clear();
            }
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("Filter:");
            ui.radio_value(&mut self.filter_mode, FilterMode::All, "All");
            ui.radio_value(&mut self.filter_mode, FilterMode::Owned, "Owned");
            ui.radio_value(&mut self.filter_mode, FilterMode::Missing, "Missing");
            ui.radio_value(
                &mut self.filter_mode,
                FilterMode::BobbinsOnly,
                "Bobbins only",
            );
            ui.radio_value(&mut self.filter_mode, FilterMode::SkeinsOnly, "Skeins only");
            ui.radio_value(&mut self.filter_mode, FilterMode::LowStock, "Low stock");
            ui.radio_value(&mut self.filter_mode, FilterMode::WithNotes, "With notes");

            if ui.button("Reset").clicked() {
                self.search.clear();
                self.filter_mode = FilterMode::All;
            }
        });
    }

    fn ui_color_list(&mut self, ui: &mut egui::Ui) {
        let query = self.search.trim().to_lowercase();
        let mut visible: Vec<DmcColor> = Vec::new();

        for color in &self.colors {
            let entry = self.entry_for(&color.code);
            let owned = is_owned(&entry);
            let notes = entry.notes.trim().to_lowercase();

            let matches_query = query.is_empty()
                || color.code.to_lowercase().contains(&query)
                || color.name.to_lowercase().contains(&query)
                || notes.contains(&query);

            let matches_filter = match self.filter_mode {
                FilterMode::All => true,
                FilterMode::Owned => owned,
                FilterMode::Missing => !owned,
                FilterMode::BobbinsOnly => entry.bobbins > 0 && entry.skeins == 0,
                FilterMode::SkeinsOnly => entry.skeins > 0 && entry.bobbins == 0,
                FilterMode::LowStock => owned && entry.total_units() <= 1,
                FilterMode::WithNotes => !entry.notes.trim().is_empty(),
            };

            if matches_query && matches_filter {
                visible.push(color.clone());
            }
        }

        ui.label(format!(
            "Showing {} of {} colours — {}",
            visible.len(),
            self.colors.len(),
            self.filter_mode.label()
        ));
        ui.label(
            "Scroll the colour list with the mouse wheel/touchpad, or drag inside the list area.",
        );
        ui.separator();

        let mut action: Option<(String, &'static str, i32)> = None;
        let mut select_code: Option<String> = None;

        egui::ScrollArea::both()
            .id_source("flosskeeper_color_list_scroll")
            .auto_shrink([false, false])
            .max_height(ui.available_height())
            .show(ui, |ui| {
                egui::Grid::new("floss_grid")
                    .striped(true)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.strong("Color");
                        ui.strong("DMC");
                        ui.strong("Name");
                        ui.strong("Bobbins");
                        ui.strong("Skeins");
                        ui.strong("Total");
                        ui.strong("Notes");
                        ui.end_row();

                        for color in visible {
                            let entry = self.entry_for(&color.code);

                            color_swatch(ui, color.rgb, [28.0, 18.0]);

                            if ui
                                .selectable_label(
                                    self.selected_code.as_deref() == Some(color.code.as_str()),
                                    &color.code,
                                )
                                .clicked()
                            {
                                select_code = Some(color.code.clone());
                            }

                            ui.label(&color.name);

                            ui.horizontal(|ui| {
                                if ui.small_button("-").clicked() {
                                    action = Some((color.code.clone(), "bobbins", -1));
                                }
                                ui.label(entry.bobbins.to_string());
                                if ui.small_button("+").clicked() {
                                    action = Some((color.code.clone(), "bobbins", 1));
                                }
                            });

                            ui.horizontal(|ui| {
                                if ui.small_button("-").clicked() {
                                    action = Some((color.code.clone(), "skeins", -1));
                                }
                                ui.label(entry.skeins.to_string());
                                if ui.small_button("+").clicked() {
                                    action = Some((color.code.clone(), "skeins", 1));
                                }
                            });

                            ui.label(entry.total_units().to_string());
                            ui.label(short_note(&entry.notes));
                            ui.end_row();
                        }
                    });
            });

        if let Some(code) = select_code {
            self.selected_code = Some(code);
        }

        if let Some((code, kind, delta)) = action {
            match kind {
                "bobbins" => self.adjust_bobbins(&code, delta),
                "skeins" => self.adjust_skeins(&code, delta),
                _ => {}
            }
        }
    }

    fn ui_selected_details(&mut self, ui: &mut egui::Ui) {
        ui.heading("Selected colour");
        ui.separator();

        let Some(color) = self.selected_color().cloned() else {
            ui.label("Select a DMC colour to edit notes.");
            return;
        };

        color_swatch(ui, color.rgb, [80.0, 50.0]);
        ui.label(RichText::new(format!("DMC {}", color.code)).strong());
        ui.label(color.name);
        ui.label(color.hex);
        ui.separator();

        let mut changed = false;
        let entry = self.entry_for_mut(&color.code);

        ui.horizontal(|ui| {
            ui.label("Bobbins:");
            if ui.button("-").clicked() {
                entry.bobbins = adjust_u32(entry.bobbins, -1);
                changed = true;
            }
            ui.label(entry.bobbins.to_string());
            if ui.button("+").clicked() {
                entry.bobbins = adjust_u32(entry.bobbins, 1);
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Skeins:");
            if ui.button("-").clicked() {
                entry.skeins = adjust_u32(entry.skeins, -1);
                changed = true;
            }
            ui.label(entry.skeins.to_string());
            if ui.button("+").clicked() {
                entry.skeins = adjust_u32(entry.skeins, 1);
                changed = true;
            }
        });

        ui.label("Notes:");
        if ui.text_edit_multiline(&mut entry.notes).changed() {
            changed = true;
        }

        if changed {
            self.dirty = true;
        }

        let code = color.code.clone();
        self.cleanup_entry_if_empty(&code);
    }
}

impl FlossKeeperApp {
    fn ui_backup_restore(&mut self, ui: &mut egui::Ui) {
        ui.heading("Backup / Restore");
        ui.label("Export or restore your FlossKeeper stash.");

        ui.add_space(10.0);

        if ui.button("Export JSON Backup").clicked() {
            self.backup_status = match backup::export_json_backup() {
                Ok(msg) => msg,
                Err(err) => format!("Export failed:\n{err}"),
            };
        }

        if ui.button("Restore JSON Backup").clicked() {
            self.backup_status = match backup::restore_json_backup() {
                Ok(msg) => format!("{msg}\n\nRestart FlossKeeper to reload the restored stash."),
                Err(err) => format!("Restore failed:\n{err}"),
            };
        }

        if ui.button("Export Plain TSV Copy").clicked() {
            self.backup_status = match backup::export_plain_tsv_copy() {
                Ok(msg) => msg,
                Err(err) => format!("TSV export failed:\n{err}"),
            };
        }

        ui.add_space(10.0);

        if !self.backup_status.is_empty() {
            ui.separator();
            ui.label(&self.backup_status);
        }
    }
}

impl FlossKeeperApp {
    fn ui_missing_colors(&mut self, ui: &mut egui::Ui) {
        ui.heading("Missing Colors");

        let total_colors = self.colors.len();
        let owned_count = self
            .colors
            .iter()
            .filter(|color| is_owned(&self.entry_for(&color.code)))
            .count();
        let missing_count = total_colors.saturating_sub(owned_count);

        ui.label(format!("Owned colors: {owned_count}"));
        ui.label(format!("Missing colors: {missing_count}"));
        ui.label(format!("Total DMC colors in app: {total_colors}"));

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Export Missing Colors TSV").clicked() {
                let contents = self.missing_colors_tsv();

                self.missing_status = match backup::export_text_file(
                    "Export Missing Colors TSV",
                    "flosskeeper_missing_colors.tsv",
                    "TSV file",
                    &["tsv"],
                    &contents,
                ) {
                    Ok(msg) => msg,
                    Err(err) => format!("Missing colors TSV export failed:\n{err}"),
                };
            }

            if ui.button("Export Missing Colors Plain Text").clicked() {
                let contents = self.missing_colors_plain_text();

                self.missing_status = match backup::export_text_file(
                    "Export Missing Colors Plain Text",
                    "flosskeeper_missing_colors.txt",
                    "Text file",
                    &["txt"],
                    &contents,
                ) {
                    Ok(msg) => msg,
                    Err(err) => format!("Missing colors text export failed:\n{err}"),
                };
            }
        });

        if !self.missing_status.is_empty() {
            ui.label(&self.missing_status);
        }

        ui.add_space(10.0);

        if missing_count == 0 {
            ui.separator();
            ui.label("You currently own every DMC color in the app list.");
            return;
        }

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("missing_colors_grid")
                .striped(true)
                .min_col_width(90.0)
                .show(ui, |ui| {
                    ui.strong("Code");
                    ui.strong("Name");
                    ui.strong("Color");
                    ui.end_row();

                    for color in &self.colors {
                        let entry = self.entry_for(&color.code);

                        if !is_owned(&entry) {
                            ui.label(&color.code);
                            ui.label(&color.name);
                            color_swatch(ui, color.rgb, [36.0, 18.0]);
                            ui.end_row();
                        }
                    }
                });
        });
    }
}

impl FlossKeeperApp {
    fn missing_colors_owned_count(&self) -> usize {
        self.colors
            .iter()
            .filter(|color| is_owned(&self.entry_for(&color.code)))
            .count()
    }

    fn missing_colors_count(&self) -> usize {
        self.colors
            .len()
            .saturating_sub(self.missing_colors_owned_count())
    }

    fn missing_colors_tsv(&self) -> String {
        let mut out = String::new();

        out.push_str("Code\tName\tHex\n");

        for color in &self.colors {
            let entry = self.entry_for(&color.code);

            if !is_owned(&entry) {
                let hex = format!(
                    "#{:02X}{:02X}{:02X}",
                    color.rgb[0], color.rgb[1], color.rgb[2]
                );

                out.push_str(&format!("{}\t{}\t{}\n", color.code, color.name, hex));
            }
        }

        out
    }

    fn missing_colors_plain_text(&self) -> String {
        let owned_count = self.missing_colors_owned_count();
        let missing_count = self.missing_colors_count();
        let total_colors = self.colors.len();

        let mut out = String::new();

        out.push_str("FlossKeeper Missing Colors Shopping List\n");
        out.push_str("=======================================\n\n");
        out.push_str(&format!("Owned colors: {owned_count}\n"));
        out.push_str(&format!("Missing colors: {missing_count}\n"));
        out.push_str(&format!("Total DMC colors in app: {total_colors}\n\n"));

        if missing_count == 0 {
            out.push_str("You currently own every DMC color in the app list.\n");
            return out;
        }

        out.push_str("Missing DMC Colors\n");
        out.push_str("------------------\n");

        for color in &self.colors {
            let entry = self.entry_for(&color.code);

            if !is_owned(&entry) {
                out.push_str(&format!("DMC {} - {}\n", color.code, color.name));
            }
        }

        out
    }
}

impl FlossKeeperApp {
    fn ui_about(&mut self, ui: &mut egui::Ui) {
        ui.heading(format!("FlossKeeper v{}", APP_VERSION));

        ui.add_space(8.0);

        ui.label("Cross-stitch floss stash tracker.");
        ui.label("Tracks bobbins, skeins, owned colors, missing colors, backups, and printable shopping lists.");

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        ui.heading("Features");

        ui.label("• Track DMC floss by bobbins and skeins");
        ui.label("• Search and filter your collection");
        ui.label("• Handle Blanc / White correctly");
        ui.label("• View missing DMC colors");
        ui.label("• Export missing colors as TSV or printable plain text");
        ui.label("• Backup and restore your stash using JSON");
        ui.label("• Export a plain TSV copy of your stash");

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        ui.heading("Data location");

        ui.monospace(self.save_path.display().to_string());

        ui.add_space(12.0);

        ui.label("FlossKeeper saves your stash locally on this computer.");
    }
}

impl FlossKeeperApp {
    fn find_color_by_code(&self, raw_code: &str) -> Option<&DmcColor> {
        let candidate = canonical_code(raw_code.trim());

        if candidate.is_empty() {
            return None;
        }

        let candidate_is_numeric = candidate.chars().all(|ch| ch.is_ascii_digit());
        let candidate_number = if candidate_is_numeric {
            let trimmed = candidate.trim_start_matches('0');
            Some(if trimmed.is_empty() { "0" } else { trimmed })
        } else {
            None
        };

        for color in &self.colors {
            let color_code = canonical_code(&color.code);

            if color_code.eq_ignore_ascii_case(&candidate) {
                return Some(color);
            }

            if let Some(candidate_number) = candidate_number {
                if color_code.chars().all(|ch| ch.is_ascii_digit()) {
                    let color_number = color_code.trim_start_matches('0');
                    let color_number = if color_number.is_empty() {
                        "0"
                    } else {
                        color_number
                    };

                    if color_number == candidate_number {
                        return Some(color);
                    }
                }
            }
        }

        None
    }

    fn pattern_required_codes(&self) -> Vec<String> {
        let mut found = std::collections::BTreeSet::new();

        for token in self
            .pattern_input
            .split(|ch: char| !ch.is_ascii_alphanumeric())
        {
            let token = token.trim();

            if token.is_empty() {
                continue;
            }

            if let Some(color) = self.find_color_by_code(token) {
                found.insert(color.code.clone());
            }
        }

        found.into_iter().collect()
    }

    fn pattern_planner_report(&self) -> String {
        let required_codes = self.pattern_required_codes();

        let mut owned_count = 0usize;
        let mut missing_count = 0usize;

        for code in &required_codes {
            if let Some(color) = self.find_color_by_code(code) {
                let entry = self.entry_for(&color.code);

                if is_owned(&entry) {
                    owned_count += 1;
                } else {
                    missing_count += 1;
                }
            }
        }

        let mut out = String::new();

        out.push_str("FlossKeeper Pattern Planner Report\n");
        out.push_str("==================================\n\n");
        out.push_str(&format!("Required colors: {}\n", required_codes.len()));
        out.push_str(&format!("Owned colors: {owned_count}\n"));
        out.push_str(&format!("Need to buy: {missing_count}\n\n"));

        if required_codes.is_empty() {
            out.push_str("No recognized DMC colors were entered.\n");
            return out;
        }

        out.push_str("Need to Buy\n");
        out.push_str("-----------\n");

        let mut wrote_missing = false;

        for code in &required_codes {
            if let Some(color) = self.find_color_by_code(code) {
                let entry = self.entry_for(&color.code);

                if !is_owned(&entry) {
                    wrote_missing = true;
                    out.push_str(&format!("DMC {} - {}\n", color.code, color.name));
                }
            }
        }

        if !wrote_missing {
            out.push_str("Nothing. You own every required color.\n");
        }

        out.push_str("\nFull Detail\n");
        out.push_str("-----------\n");

        for code in &required_codes {
            if let Some(color) = self.find_color_by_code(code) {
                let entry = self.entry_for(&color.code);
                let status = if is_owned(&entry) { "OWNED" } else { "MISSING" };

                out.push_str(&format!(
                    "DMC {} - {} | {} | bobbins: {} | skeins: {}\n",
                    color.code, color.name, status, entry.bobbins, entry.skeins
                ));
            }
        }

        out
    }

    fn pattern_need_to_buy_plain_text(&self) -> String {
        let required_codes = self.pattern_required_codes();

        let mut out = String::new();

        for code in &required_codes {
            if let Some(color) = self.find_color_by_code(code) {
                let entry = self.entry_for(&color.code);

                if !is_owned(&entry) {
                    out.push_str(&format!("DMC {} - {}\n", color.code, color.name));
                }
            }
        }

        if out.trim().is_empty() {
            "Nothing to buy. You own every required color.".to_string()
        } else {
            out
        }
    }

    fn ui_pattern_planner(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.heading("Pattern Planner");
                ui.label("Paste the DMC colors required by a pattern.");
                ui.label("Best input: one code per line, or codes separated by commas/spaces.");

                ui.add_space(8.0);

                ui.add_sized(
                    [ui.available_width(), 140.0],
                    egui::TextEdit::multiline(&mut self.pattern_input)
                        .hint_text("310\n666\nBlanc\nB5200\n3847"),
                );

                ui.horizontal(|ui| {
                    if ui.button("Clear").clicked() {
                        self.pattern_input.clear();
                        self.pattern_status.clear();
                    }

                    if ui.button("Export Pattern Report").clicked() {
                        let contents = self.pattern_planner_report();

                        self.pattern_status = match backup::export_text_file(
                            "Export Pattern Planner Report",
                            "flosskeeper_pattern_report.txt",
                            "Text file",
                            &["txt"],
                            &contents,
                        ) {
                            Ok(msg) => msg,
                            Err(err) => format!("Pattern report export failed:\n{err}"),
                        };
                    }

                    if ui.button("Copy Need-to-Buy List").clicked() {
                        let contents = self.pattern_need_to_buy_plain_text();
                        ui.ctx().copy_text(contents);
                        self.pattern_status = "Need-to-buy list copied to clipboard.".to_string();
                    }
                });

                if !self.pattern_status.is_empty() {
                    ui.label(&self.pattern_status);
                }

                ui.add_space(10.0);

                let required_codes = self.pattern_required_codes();

                if required_codes.is_empty() {
                    ui.separator();
                    ui.label("No recognized DMC colors yet.");
                    ui.label("Tip: paste codes like 310, 666, Blanc, B5200, 3847.");
                    return;
                }

                let mut owned_count = 0usize;
                let mut missing_count = 0usize;

                for code in &required_codes {
                    if let Some(color) = self.find_color_by_code(code) {
                        let entry = self.entry_for(&color.code);

                        if is_owned(&entry) {
                            owned_count += 1;
                        } else {
                            missing_count += 1;
                        }
                    }
                }

                ui.separator();
                ui.label(format!("Required colors: {}", required_codes.len()));
                ui.label(format!("Owned colors: {owned_count}"));
                ui.label(format!("Need to buy: {missing_count}"));

                ui.add_space(10.0);

                egui::Grid::new("pattern_planner_grid")
                    .striped(true)
                    .min_col_width(80.0)
                    .show(ui, |ui| {
                        ui.strong("Status");
                        ui.strong("Code");
                        ui.strong("Name");
                        ui.strong("Bobbins");
                        ui.strong("Skeins");
                        ui.strong("Color");
                        ui.end_row();

                        for code in &required_codes {
                            if let Some(color) = self.find_color_by_code(code) {
                                let entry = self.entry_for(&color.code);
                                let status = if is_owned(&entry) { "OWNED" } else { "MISSING" };

                                ui.label(status);
                                ui.label(&color.code);
                                ui.label(&color.name);
                                ui.label(entry.bobbins.to_string());
                                ui.label(entry.skeins.to_string());
                                color_swatch(ui, color.rgb, [36.0, 18.0]);
                                ui.end_row();
                            }
                        }
                    });
            });
    }
}

impl eframe::App for FlossKeeperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_first_run_help_window(ctx);
        self.show_about_window(ctx);

        ctx.set_visuals(egui::Visuals::light());

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Help").clicked() {
                    self.show_first_run_help = true;
                }

                if ui.button(format!("About v{}", APP_VERSION)).clicked() {
                    self.show_about_window = true;
                }
            });
            ui.separator();

            self.ui_top(ui);

            ui.separator();

            ui.horizontal(|ui| {
                ui.label(format!("FlossKeeper v{}", APP_VERSION));

                ui.separator();

                ui.selectable_value(&mut self.active_tab, AppTab::Collection, "Collection");

                ui.selectable_value(
                    &mut self.active_tab,
                    AppTab::MissingColors,
                    "Missing Colors",
                );

                ui.selectable_value(
                    &mut self.active_tab,
                    AppTab::PatternPlanner,
                    "Pattern Planner",
                );

                ui.selectable_value(
                    &mut self.active_tab,
                    AppTab::BackupRestore,
                    "Backup / Restore",
                );

                ui.selectable_value(&mut self.active_tab, AppTab::About, "About");
            });
        });

        match self.active_tab {
            AppTab::Collection => {
                egui::SidePanel::right("details_panel")
                    .default_width(260.0)
                    .show(ctx, |ui| {
                        self.ui_selected_details(ui);
                    });

                egui::CentralPanel::default().show(ctx, |ui| {
                    self.ui_filters(ui);
                    self.ui_color_list(ui);
                });
            }
            AppTab::BackupRestore => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.ui_backup_restore(ui);
                });
            }
            AppTab::MissingColors => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.ui_missing_colors(ui);
                });
            }
            AppTab::PatternPlanner => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.ui_pattern_planner(ui);
                });
            }
            AppTab::About => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.ui_about(ui);
                });
            }
        }
    }
}

fn is_owned(entry: &InventoryEntry) -> bool {
    entry.bobbins > 0 || entry.skeins > 0
}

fn short_note(note: &str) -> String {
    let note = note.trim();
    let mut chars = note.chars();
    let shortened: String = chars.by_ref().take(32).collect();

    if chars.next().is_some() {
        format!("{shortened}…")
    } else {
        shortened
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn adjust_u32(value: u32, delta: i32) -> u32 {
    if delta < 0 {
        value.saturating_sub(delta.unsigned_abs())
    } else {
        value.saturating_add(delta as u32)
    }
}

fn color_swatch(ui: &mut egui::Ui, rgb: [u8; 3], size: [f32; 2]) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size[0], size[1]), egui::Sense::hover());
    let color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
    ui.painter().rect_filled(rect, 3.0, color);
    ui.painter()
        .rect_stroke(rect, 3.0, egui::Stroke::new(1.0, Color32::BLACK));
}

fn load_dmc_colors() -> Vec<DmcColor> {
    let mut colors = Vec::new();

    for (i, raw_line) in DMC_CSV.lines().enumerate() {
        if i == 0 {
            continue;
        }

        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(color) = parse_dmc_line(line) {
            colors.push(color);
        }
    }

    colors.sort_by(|a, b| smart_code_key(&a.code).cmp(&smart_code_key(&b.code)));
    colors
}

fn canonical_code(code: &str) -> String {
    let trimmed = code.trim();
    if trimmed.eq_ignore_ascii_case("White") {
        "Blanc".to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_dmc_line(line: &str) -> Option<DmcColor> {
    let first = line.find(',')?;
    let last = line.rfind(',')?;
    if first == last {
        return None;
    }

    let raw_code = line[..first].trim();
    if raw_code.eq_ignore_ascii_case("White") {
        return None;
    }

    let code = canonical_code(raw_code);
    let mut name = line[first + 1..last].trim().to_string();
    if code == "Blanc" {
        name = "Blanc / White".to_string();
    }

    let hex = line[last + 1..].trim().trim_matches('"').to_string();
    let rgb = parse_hex_rgb(&hex)?;

    Some(DmcColor {
        code,
        name,
        hex,
        rgb,
    })
}

fn parse_hex_rgb(hex: &str) -> Option<[u8; 3]> {
    let value = hex.trim().trim_start_matches('#');
    if value.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&value[0..2], 16).ok()?;
    let g = u8::from_str_radix(&value[2..4], 16).ok()?;
    let b = u8::from_str_radix(&value[4..6], 16).ok()?;
    Some([r, g, b])
}

fn smart_code_key(code: &str) -> (u8, u32, String) {
    if let Ok(n) = code.parse::<u32>() {
        return (0, n, code.to_string());
    }

    (1, 0, code.to_lowercase())
}

fn default_save_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Path::new(&appdata)
                .join("FlossKeeper")
                .join("flosskeeper_collection.tsv");
        }
    }

    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        return Path::new(&config_home)
            .join("flosskeeper")
            .join("flosskeeper_collection.tsv");
    }

    if let Ok(home) = std::env::var("HOME") {
        return Path::new(&home)
            .join(".config")
            .join("flosskeeper")
            .join("flosskeeper_collection.tsv");
    }

    PathBuf::from("flosskeeper_collection.tsv")
}

fn load_app_icon() -> std::sync::Arc<egui::IconData> {
    let icon_bytes = include_bytes!("../assets/icons/flosskeeper.png");

    eframe::icon_data::from_png_bytes(icon_bytes)
        .expect("Failed to load FlossKeeper window icon")
        .into()
}

fn default_first_run_help_path() -> PathBuf {
    let mut path = default_save_path();
    path.set_file_name("first_run_help_dismissed.txt");
    path
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_app_id("com.jesterace.FlossKeeper")
            .with_title("FlossKeeper")
            .with_inner_size([1000.0, 700.0])
            .with_icon(load_app_icon()),
        ..Default::default()
    };

    eframe::run_native(
        &format!("FlossKeeper v{}", APP_VERSION),
        options,
        Box::new(|_cc| Box::<FlossKeeperApp>::default()),
    )
}
