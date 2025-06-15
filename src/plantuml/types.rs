/// Arrow types in `PlantUML` - used only during parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowType {
    SolidSync,           // ->
    SolidAsync,          // ->>
    DashedSync,          // -->
    DashedAsync,         // -->>
    LeftSync,            // <-
    LeftAsync,           // <<-
    LeftDashedSync,      // <--
    LeftDashedAsync,     // <<--
    BiDirectional,       // <->
    BiDashedDirectional, // <-->
    Lost,                // -\
    Found,               // \-
    SelfCall,            // \\
}

impl ArrowType {
    pub fn parse_arrow(s: &str) -> Option<Self> {
        match s {
            "->" => Some(Self::SolidSync),
            "->>" => Some(Self::SolidAsync),
            "-->" => Some(Self::DashedSync),
            "-->>" => Some(Self::DashedAsync),
            "<-" => Some(Self::LeftSync),
            "<<-" => Some(Self::LeftAsync),
            "<--" => Some(Self::LeftDashedSync),
            "<<--" => Some(Self::LeftDashedAsync),
            "<->" => Some(Self::BiDirectional),
            "<-->" => Some(Self::BiDashedDirectional),
            "-\\" => Some(Self::Lost),
            "\\-" => Some(Self::Found),
            "\\\\" => Some(Self::SelfCall),
            _ => None,
        }
    }

    pub fn to_message_type(self) -> crate::events::MessageType {
        match self {
            Self::SolidSync | Self::LeftSync | Self::BiDirectional => {
                crate::events::MessageType::Synchronous
            }
            Self::SolidAsync | Self::LeftAsync => crate::events::MessageType::Asynchronous,
            Self::DashedSync
            | Self::DashedAsync
            | Self::LeftDashedSync
            | Self::LeftDashedAsync
            | Self::BiDashedDirectional => crate::events::MessageType::Return,
            Self::Lost | Self::Found | Self::SelfCall => crate::events::MessageType::Synchronous,
        }
    }

    pub fn is_reversed(self) -> bool {
        matches!(
            self,
            Self::LeftSync
                | Self::LeftAsync
                | Self::LeftDashedSync
                | Self::LeftDashedAsync
                | Self::Found
        )
    }
}
