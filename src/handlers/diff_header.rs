use crate::{features, utils};
    Added,
    Removed,
    /// Check for the old mode|new mode lines and cache their info for later use.
    pub fn handle_diff_header_mode_line(&mut self) -> std::io::Result<bool> {
        let mut handled_line = false;
        if let Some(line_suf) = self.line.strip_prefix("old mode ") {
            self.state = State::DiffHeader(DiffType::Unified);
            if self.should_handle() && !self.config.color_only {
                self.mode_info = line_suf.to_string();
                handled_line = true;
            }
        } else if let Some(line_suf) = self.line.strip_prefix("new mode ") {
            self.state = State::DiffHeader(DiffType::Unified);
            if self.should_handle() && !self.config.color_only && !self.mode_info.is_empty() {
                self.mode_info = match (self.mode_info.as_str(), line_suf) {
                    // 100755 for executable and 100644 for non-executable are the only file modes Git records.
                    // https://medium.com/@tahteche/how-git-treats-changes-in-file-permissions-f71874ca239d
                    ("100644", "100755") => "mode +x".to_string(),
                    ("100755", "100644") => "mode -x".to_string(),
                    _ => format!(
                        "mode {} {} {}",
                        self.mode_info, self.config.right_arrow, line_suf
                    ),
                };
                handled_line = true;
            }
        }
        Ok(handled_line)
    }

    fn should_write_generic_diff_header_header_line(&mut self) -> std::io::Result<bool> {
        // In color_only mode, raw_line's structure shouldn't be changed.
        // So it needs to avoid fn _handle_diff_header_header_line
        // (it connects the plus_file and minus_file),
        // and to call fn handle_generic_diff_header_header_line directly.
        if self.config.color_only {
            write_generic_diff_header_header_line(
                &self.line,
                &self.raw_line,
                &mut self.painter,
                &mut self.mode_info,
                self.config,
            )?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

                || self.line.starts_with("copy from "))
    /// Check for and handle the "--- filename ..." line.
        let (path_or_mode, file_event) =
            parse_diff_header_line(&self.line, self.source == Source::GitDiff);

        self.minus_file = utils::path::relativize_path_maybe(&path_or_mode, self.config)
            .map(|p| p.to_string_lossy().to_owned().to_string())
            .unwrap_or(path_or_mode);
        self.painter.paint_buffered_minus_and_plus_lines();
        self.should_write_generic_diff_header_header_line()
                || self.line.starts_with("copy to "))
    /// Check for and handle the "+++ filename ..." line.
        let (path_or_mode, file_event) =
            parse_diff_header_line(&self.line, self.source == Source::GitDiff);

        self.plus_file = utils::path::relativize_path_maybe(&path_or_mode, self.config)
            .map(|p| p.to_string_lossy().to_owned().to_string())
            .unwrap_or(path_or_mode);
        self.painter.paint_buffered_minus_and_plus_lines();
        if self.should_write_generic_diff_header_header_line()? {
            handled_line = true;
            self.handled_diff_header_header_line_file_pair = self.current_file_pair.clone();
        }
        Ok(handled_line)
    }

    #[inline]
    fn test_diff_header_file_operation_line(&self) -> bool {
        (matches!(self.state, State::DiffHeader(_)) || self.source == Source::DiffUnified)
            && (self.line.starts_with("deleted file mode ")
                || self.line.starts_with("new file mode "))
    }

    /// Check for and handle the "deleted file ..."  line.
    pub fn handle_diff_header_file_operation_line(&mut self) -> std::io::Result<bool> {
        if !self.test_diff_header_file_operation_line() {
            return Ok(false);
        }
        let mut handled_line = false;
        let (_mode_info, file_event) =
            parse_diff_header_line(&self.line, self.source == Source::GitDiff);
        let name = get_repeated_file_path_from_diff_line(&self.diff_line)
            .unwrap_or_else(|| "".to_string());
        match file_event {
            FileEvent::Removed => {
                self.minus_file = name;
                self.plus_file = "/dev/null".into();
                self.minus_file_event = FileEvent::Change;
                self.plus_file_event = FileEvent::Change;
                self.current_file_pair = Some((self.minus_file.clone(), self.plus_file.clone()));
            }
            FileEvent::Added => {
                self.minus_file = "/dev/null".into();
                self.plus_file = name;
                self.minus_file_event = FileEvent::Change;
                self.plus_file_event = FileEvent::Change;
                self.current_file_pair = Some((self.minus_file.clone(), self.plus_file.clone()));
            }
            _ => (),
        }

        if self.should_write_generic_diff_header_header_line()?
            || (self.should_handle()
                && self.handled_diff_header_header_line_file_pair != self.current_file_pair)
        {
            handled_line = true;
        write_generic_diff_header_header_line(
            &line,
            &line,
            &mut self.painter,
            &mut self.mode_info,
            self.config,
        )
    }

    #[inline]
    fn test_pending_line_with_diff_name(&self) -> bool {
        matches!(self.state, State::DiffHeader(_)) || self.source == Source::DiffUnified
    }

    pub fn handle_pending_line_with_diff_name(&mut self) -> std::io::Result<()> {
        if !self.test_pending_line_with_diff_name() {
            return Ok(());
        }

        if !self.mode_info.is_empty() {
            let format_label = |label: &str| {
                if !label.is_empty() {
                    format!("{} ", label)
                } else {
                    "".to_string()
                }
            };
            let format_file = |file| match (
                self.config.hyperlinks,
                utils::path::absolute_path(file, self.config),
            ) {
                (true, Some(absolute_path)) => features::hyperlinks::format_osc8_file_hyperlink(
                    absolute_path,
                    None,
                    file,
                    self.config,
                ),
                _ => Cow::from(file),
            };
            let label = format_label(&self.config.file_modified_label);
            let name = get_repeated_file_path_from_diff_line(&self.diff_line)
                .unwrap_or_else(|| "".to_string());
            let line = format!("{}{}", label, format_file(&name));
            write_generic_diff_header_header_line(
                &line,
                &line,
                &mut self.painter,
                &mut self.mode_info,
                self.config,
            )
        } else if !self.config.color_only
            && self.should_handle()
            && self.handled_diff_header_header_line_file_pair != self.current_file_pair
        {
            self._handle_diff_header_header_line(self.source == Source::DiffUnified)?;
            self.handled_diff_header_header_line_file_pair = self.current_file_pair.clone();
            Ok(())
        } else {
            Ok(())
        }
    mode_info: &mut String,
        mode_info,
    if !mode_info.is_empty() {
        mode_info.truncate(0);
    }
fn parse_diff_header_line(line: &str, git_diff_name: bool) -> (String, FileEvent) {
    match line {
        line if line.starts_with("new file mode ") => {
            (line[14..].to_string(), FileEvent::Added) // "new file mode ".len()
        line if line.starts_with("deleted file mode ") => {
            (line[18..].to_string(), FileEvent::Removed) // "deleted file mode ".len()
            let formatted_file = if let Some(regex_replacement) = &config.file_regex_replacement {
                regex_replacement.execute(file)
            };
            match (config.hyperlinks, utils::path::absolute_path(file, config)) {
                (true, Some(absolute_path)) => features::hyperlinks::format_osc8_file_hyperlink(
                    absolute_path,
                    None,
                    &formatted_file,
                    config,
                ),
                _ => formatted_file,
            // minus_file_event == plus_file_event
            parse_diff_header_line("--- /dev/null", true),
                parse_diff_header_line(&format!("--- {}src/delta.rs", prefix), true),
            parse_diff_header_line("--- src/delta.rs", true),
            parse_diff_header_line("+++ src/delta.rs", true),
            parse_diff_header_line("+++ a/my src/delta.rs", true),
            parse_diff_header_line("+++ my src/delta.rs", true),
            parse_diff_header_line("+++ a/src/my delta.rs", true),
            parse_diff_header_line("+++ a/my src/my delta.rs", true),
            parse_diff_header_line("+++ b/my src/my enough/my delta.rs", true),
            parse_diff_header_line("rename from nospace/file2.el", true),
            parse_diff_header_line("rename from with space/file1.el", true),
            parse_diff_header_line("--- src/delta.rs", false),
            parse_diff_header_line("+++ src/delta.rs", false),
            get_repeated_file_path_from_diff_line(
                "diff --git a/.config/Code - Insiders/User/settings.json b/.config/Code - Insiders/User/settings.json"),
            Some(".config/Code - Insiders/User/settings.json".to_string())
        );