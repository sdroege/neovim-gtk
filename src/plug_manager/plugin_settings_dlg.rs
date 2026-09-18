use futures::channel::oneshot;

use gtk::prelude::*;

use super::store;

pub struct Builder<'a> {
    title: &'a str,
}

impl<'a> Builder<'a> {
    pub fn new(title: &'a str) -> Self {
        Builder { title }
    }

    pub async fn show<F: IsA<gtk::Window>>(&self, parent: &F) -> Option<store::PlugInfo> {
        let dlg = gtk::Window::builder()
            .title(self.title)
            .transient_for(parent)
            .modal(true)
            .default_width(400)
            .build();

        let header_bar = gtk::HeaderBar::builder().build();
        let cancel_btn = gtk::Button::with_label("Cancel");
        let ok_btn = gtk::Button::with_label("Ok");
        ok_btn.add_css_class("suggested-action");
        header_bar.pack_start(&cancel_btn);
        header_bar.pack_end(&ok_btn);
        dlg.set_titlebar(Some(&header_bar));

        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        let border = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .build();

        let path = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(5)
            .margin_start(5)
            .margin_bottom(5)
            .margin_top(5)
            .margin_end(5)
            .build();
        let path_lbl = gtk::Label::new(Some("Repo"));
        let path_e = gtk::Entry::new();
        path_e.set_placeholder_text(Some("user_name/repo_name"));

        path.append(&path_lbl);
        path.append(&path_e);

        list.append(&path);

        let name = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(5)
            .margin_start(5)
            .margin_end(5)
            .margin_top(5)
            .margin_bottom(5)
            .build();
        let name_lbl = gtk::Label::new(Some("Name"));
        let name_e = gtk::Entry::new();

        name.append(&name_lbl);
        name.append(&name_e);

        list.append(&name);

        border.append(&list);
        content.append(&border);
        dlg.set_child(Some(&content));

        path_e.connect_changed(glib::clone!(
            #[strong]
            name_e,
            move |p| {
                if let Some(name) = extract_name(p.text().as_str()) {
                    name_e.set_text(&name);
                }
            }
        ));

        let (sender, receiver) = oneshot::channel::<bool>();
        let sender = std::rc::Rc::new(std::cell::RefCell::new(Some(sender)));

        ok_btn.connect_clicked(glib::clone!(
            #[strong]
            sender,
            move |_| {
                if let Some(sender) = sender.borrow_mut().take() {
                    let _ = sender.send(true);
                }
            }
        ));

        cancel_btn.connect_clicked(glib::clone!(
            #[strong]
            sender,
            move |_| {
                if let Some(sender) = sender.borrow_mut().take() {
                    let _ = sender.send(false);
                }
            }
        ));

        dlg.connect_close_request(glib::clone!(
            #[strong]
            sender,
            move |_| {
                if let Some(sender) = sender.borrow_mut().take() {
                    let _ = sender.send(false);
                }
                glib::Propagation::Proceed
            }
        ));

        dlg.set_visible(true);

        let res = receiver.await.unwrap_or(false);

        let res = if res {
            let path = path_e.text().to_string();
            let name = name_e.text();

            let name = if name.trim().is_empty() {
                match extract_name(&path) {
                    Some(name) => name,
                    None => path.clone(),
                }
            } else {
                name.to_string()
            };

            Some(store::PlugInfo::new(name, path))
        } else {
            None
        };

        dlg.close();

        res
    }
}

fn extract_name(path: &str) -> Option<String> {
    if let Some(idx) = path.rfind(['/', '\\']) {
        if idx < path.len() - 1 {
            let path = path.trim_end_matches(".git");
            Some(path[idx + 1..].to_owned())
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_name() {
        assert_eq!(
            Some("plugin_name".to_owned()),
            extract_name("http://github.com/somebody/plugin_name.git")
        );
    }
}
