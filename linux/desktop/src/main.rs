pub mod ipc_client;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;
use ipc_client::{DeviceView, IpcClient, IpcCommand};

#[derive(Clone)]
pub struct AppContext {
    pub client: Arc<IpcClient>,
}

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("NOVA-Link Desktop Application initializing (GTK4 + Libadwaita)");

    let ctx = AppContext {
        client: Arc::new(IpcClient::new()),
    };

    #[cfg(feature = "gui")]
    {
        use libadwaita::prelude::*;
        use libadwaita::{Application, ApplicationWindow, HeaderBar, PreferencesGroup, ActionRow, ViewStack};
        use gtk4::{Box as GtkBox, Button, Label, Orientation, Switch};

        let app = Application::builder()
            .application_id("com.novalink.NovaLink")
            .build();

        app.connect_activate(move |app| {
            build_ui(app, &ctx);
        });

        app.run();
    }

    #[cfg(not(feature = "gui"))]
    {
        info!("Running NOVA-Link Desktop Client in headless CLI / test mode");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let res = ctx.client.send_command(IpcCommand::ListDevices).await;
            info!("Daemon query result: {:?}", res);
        });
    }
}

#[cfg(feature = "gui")]
fn build_ui(app: &libadwaita::Application, ctx: &AppContext) {
    use libadwaita::prelude::*;
    use libadwaita::{ActionRow, ApplicationWindow, HeaderBar, PreferencesGroup, StatusPage};
    use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, ProgressBar, Switch};

    let window = ApplicationWindow::builder()
        .application(app)
        .title("NOVA-Link")
        .default_width(640)
        .default_height(600)
        .build();

    let main_box = GtkBox::new(Orientation::Vertical, 0);

    // Header Bar
    let header_bar = HeaderBar::new();
    main_box.append(&header_bar);

    // Preferences & Device Group
    let content_box = GtkBox::new(Orientation::Vertical, 16);
    content_box.set_margin_top(24);
    content_box.set_margin_bottom(24);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    let pref_group = PreferencesGroup::builder()
        .title("Connected Devices")
        .description("Nearby Android devices available over LAN")
        .build();

    // Sample Device Row: Pixel 8
    let device_row = ActionRow::builder()
        .title("Pixel 8")
        .subtitle("Connected • 192.168.1.45")
        .build();

    let send_file_btn = Button::with_label("Send File");
    send_file_btn.set_valign(Align::Center);
    device_row.add_suffix(&send_file_btn);

    let send_text_btn = Button::with_label("Send Text");
    send_text_btn.set_valign(Align::Center);
    device_row.add_suffix(&send_text_btn);

    pref_group.add(&device_row);
    content_box.append(&pref_group);

    // Privacy & Settings Group
    let settings_group = PreferencesGroup::builder()
        .title("Privacy & Synchronization")
        .build();

    let clip_row = ActionRow::builder()
        .title("Clipboard Synchronization")
        .subtitle("Automatically sync text and URLs between devices")
        .build();

    let clip_switch = Switch::new();
    clip_switch.set_active(true);
    clip_switch.set_valign(Align::Center);
    clip_row.add_suffix(&clip_switch);

    settings_group.add(&clip_row);
    content_box.append(&settings_group);

    main_box.append(&content_box);
    window.set_content(Some(&main_box));
    window.present();
}
