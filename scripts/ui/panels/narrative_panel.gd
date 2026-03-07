extends PanelContainer

const ICON_COLORS: Dictionary = {
	0: Color(0.0, 0.0, 0.0, 0.0),
	1: Color(0.20, 0.90, 0.40, 1.0),
	2: Color(0.90, 0.80, 0.20, 1.0),
	3: Color(0.40, 0.40, 0.40, 0.5),
	4: Color(0.90, 0.20, 0.20, 1.0),
}

@onready var title_label: Label = $Margin/VBox/TitleRow/TitleLabel
@onready var ai_icon: ColorRect = $Margin/VBox/TitleRow/AiIcon
@onready var personality_label: Label = $Margin/VBox/PersonalitySection/PersonalitySectionLabel
@onready var personality_text: RichTextLabel = $Margin/VBox/PersonalitySection/PersonalityText
@onready var personality_shimmer: ColorRect = $Margin/VBox/PersonalitySection/PersonalityShimmer
@onready var event_label: Label = $Margin/VBox/EventSection/EventSectionLabel
@onready var event_text: RichTextLabel = $Margin/VBox/EventSection/EventText
@onready var event_shimmer: ColorRect = $Margin/VBox/EventSection/EventShimmer
@onready var inner_label: Label = $Margin/VBox/InnerSection/InnerSectionLabel
@onready var inner_text: RichTextLabel = $Margin/VBox/InnerSection/InnerText
@onready var inner_shimmer: ColorRect = $Margin/VBox/InnerSection/InnerShimmer
@onready var disabled_overlay: ColorRect = $DisabledOverlay
@onready var disabled_label: Label = $DisabledOverlay/DisabledLabel


func render(display_data: Dictionary) -> void:
	var labels: Array = display_data.get("section_labels", ["", "", ""])
	title_label.text = str(display_data.get("panel_title", ""))
	personality_label.text = str(labels[0])
	event_label.text = str(labels[1])
	inner_label.text = str(labels[2])

	disabled_overlay.visible = bool(display_data.get("show_disabled_overlay", false))
	disabled_label.text = str(display_data.get("disabled_message", ""))

	personality_text.text = str(display_data.get("personality_text", ""))
	personality_text.visible = bool(display_data.get("show_personality", false))
	personality_shimmer.visible = bool(display_data.get("show_personality_shimmer", false))

	event_text.text = str(display_data.get("event_text", ""))
	event_text.visible = bool(display_data.get("show_event", false))
	event_shimmer.visible = bool(display_data.get("show_event_shimmer", false))

	inner_text.text = str(display_data.get("inner_text", ""))
	inner_text.visible = bool(display_data.get("show_inner", false))
	inner_shimmer.visible = bool(display_data.get("show_inner_shimmer", false))

	var icon_state: int = int(display_data.get("ai_icon_state", 0))
	ai_icon.visible = icon_state > 0
	ai_icon.color = ICON_COLORS.get(icon_state, Color(1.0, 1.0, 1.0, 1.0))
	ai_icon.tooltip_text = str(display_data.get("ai_label_tooltip", ""))
