pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Tel,
    Url,
    Search,
    Date,
    Time,
    DatetimeLocal,
    Month,
    Week,
    Color,
    File,
    Hidden,
    Checkbox,
    Radio,
    Range,
    Submit,
    Reset,
    Button,
    Image,
}

impl InputType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Password => "password",
            Self::Email => "email",
            Self::Number => "number",
            Self::Tel => "tel",
            Self::Url => "url",
            Self::Search => "search",
            Self::Date => "date",
            Self::Time => "time",
            Self::DatetimeLocal => "datetime-local",
            Self::Month => "month",
            Self::Week => "week",
            Self::Color => "color",
            Self::File => "file",
            Self::Hidden => "hidden",
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Range => "range",
            Self::Submit => "submit",
            Self::Reset => "reset",
            Self::Button => "button",
            Self::Image => "image",
        }
    }
}

pub enum LinkTarget {
    Self_,
    Blank,
    Parent,
    Top,
}

impl LinkTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Self_ => "_self",
            Self::Blank => "_blank",
            Self::Parent => "_parent",
            Self::Top => "_top",
        }
    }
}

pub enum FormMethod {
    Get,
    Post,
    Dialog,
}

impl FormMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Dialog => "dialog",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_type() {
        assert_eq!(InputType::Checkbox.as_str(), "checkbox");
        assert_eq!(InputType::Email.as_str(), "email");
        assert_eq!(InputType::DatetimeLocal.as_str(), "datetime-local");
    }

    #[test]
    fn test_link_target() {
        assert_eq!(LinkTarget::Blank.as_str(), "_blank");
        assert_eq!(LinkTarget::Self_.as_str(), "_self");
    }

    #[test]
    fn test_form_method() {
        assert_eq!(FormMethod::Post.as_str(), "post");
    }
}
