use ratatui::{
    buffer::Buffer,
    widgets::{Block, Borders, Paragraph, WidgetRef},
};

use crate::objects::{
    prelude::BodyData,
    orbiting_obj::OrbitingObjects
};

pub struct InfoWidget {
    pub body_info: BodyData,
    pub orbiting_obj: OrbitingObjects,
}

impl WidgetRef for InfoWidget {
    fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let body_info = &self.body_info;
        let orbiting_obj = &self.orbiting_obj;
        let info = Paragraph::new(format!(
            "Body type: {}\n\
            N of orbiting bodies: {}\n\
            Radius: {} km\n\
            Revolution period: {} earth days",
            body_info.body_type,
            orbiting_obj.0.len(),
            body_info.radius,
            body_info.revolution_period,
        ))
        .block(
            Block::default()
                .title(&body_info.name[..])
                .borders(Borders::ALL),
        );
        info.render_ref(area, buf);
    }
}
