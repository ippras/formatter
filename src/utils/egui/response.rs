use egui::{InnerResponse, Response};

/// Extension methods for [`Response`]
pub trait ResponseExt {
    fn with_clicked(self, f: impl Fn()) -> Response;
}

impl ResponseExt for Response {
    fn with_clicked(self, f: impl Fn()) -> Response {
        if self.clicked() {
            f()
        }
        self
    }
}

/// Extension methods for [`InnerResponse`]
pub trait InnerResponseExt {
    fn flatten(self) -> Response;
}

impl InnerResponseExt for InnerResponse<Response> {
    fn flatten(self) -> Response {
        self.inner | self.response
    }
}
