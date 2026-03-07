extends Node

const WIND_PATH: String = "res://assets/audio/wind_loop.wav"
const BIRDS_PATH: String = "res://assets/audio/birds_loop.wav"
const WATER_PATH: String = "res://assets/audio/water_loop.wav"

const BIRDS_VOLUME_OFFSET_DB: float = -3.0
const WATER_VOLUME_OFFSET_DB: float = -6.0
const WIND_MODULATION_MIN_DB: float = -3.0
const WIND_MODULATION_MAX_DB: float = 1.0
const BIRDS_MODULATION_MIN_DB: float = -4.0
const BIRDS_MODULATION_MAX_DB: float = 2.0
const WATER_MODULATION_MIN_DB: float = -2.0
const WATER_MODULATION_MAX_DB: float = 1.0
const MODULATION_INTERVAL_MIN: float = 10.0
const MODULATION_INTERVAL_MAX: float = 30.0

var _wind_player: AudioStreamPlayer
var _birds_player: AudioStreamPlayer
var _water_player: AudioStreamPlayer
var _rng: RandomNumberGenerator = RandomNumberGenerator.new()
var _is_muted: bool = false
var _base_volume_db: float = -15.0
var _modulation_timer: float = 0.0
var _next_modulation_time: float = 15.0


func _ready() -> void:
	_rng.randomize()
	process_mode = Node.PROCESS_MODE_ALWAYS
	_wind_player = _build_player("WindPlayer", WIND_PATH)
	_birds_player = _build_player("BirdsPlayer", BIRDS_PATH)
	_water_player = _build_player("WaterPlayer", WATER_PATH)
	_apply_base_volumes()
	_play_all()


func _process(delta: float) -> void:
	if _is_muted:
		return
	_modulation_timer += delta
	if _modulation_timer < _next_modulation_time:
		return
	_modulation_timer = 0.0
	_next_modulation_time = _rng.randf_range(MODULATION_INTERVAL_MIN, MODULATION_INTERVAL_MAX)
	_apply_modulated_volumes()


func toggle_mute() -> bool:
	set_muted(not _is_muted)
	return _is_muted


func set_muted(muted: bool) -> void:
	_is_muted = muted
	for player: AudioStreamPlayer in [_wind_player, _birds_player, _water_player]:
		if player == null:
			continue
		player.stream_paused = muted
		if not muted and not player.playing:
			player.play()


func is_muted() -> bool:
	return _is_muted


func set_master_volume_db(volume_db: float) -> void:
	_base_volume_db = volume_db
	_apply_base_volumes()


func _build_player(player_name: String, stream_path: String) -> AudioStreamPlayer:
	var player: AudioStreamPlayer = AudioStreamPlayer.new()
	player.name = player_name
	player.bus = &"Master"
	player.autoplay = false
	player.stream = _load_wav_stream(stream_path)
	player.finished.connect(_on_player_finished.bind(player))
	add_child(player)
	return player


func _play_all() -> void:
	for player: AudioStreamPlayer in [_wind_player, _birds_player, _water_player]:
		if player != null and player.stream != null and not player.playing:
			player.play()


func _apply_base_volumes() -> void:
	if _wind_player != null:
		_wind_player.volume_db = _base_volume_db
	if _birds_player != null:
		_birds_player.volume_db = _base_volume_db + BIRDS_VOLUME_OFFSET_DB
	if _water_player != null:
		_water_player.volume_db = _base_volume_db + WATER_VOLUME_OFFSET_DB


func _apply_modulated_volumes() -> void:
	if _wind_player != null:
		_wind_player.volume_db = _base_volume_db + _rng.randf_range(WIND_MODULATION_MIN_DB, WIND_MODULATION_MAX_DB)
	if _birds_player != null:
		_birds_player.volume_db = _base_volume_db + BIRDS_VOLUME_OFFSET_DB + _rng.randf_range(BIRDS_MODULATION_MIN_DB, BIRDS_MODULATION_MAX_DB)
	if _water_player != null:
		_water_player.volume_db = _base_volume_db + WATER_VOLUME_OFFSET_DB + _rng.randf_range(WATER_MODULATION_MIN_DB, WATER_MODULATION_MAX_DB)


func _on_player_finished(player: AudioStreamPlayer) -> void:
	if player == null or _is_muted:
		return
	if player.stream != null:
		player.play()


func _load_wav_stream(stream_path: String) -> AudioStream:
	if not FileAccess.file_exists(stream_path):
		return null
	var wav_bytes: PackedByteArray = FileAccess.get_file_as_bytes(stream_path)
	if wav_bytes.size() < 44:
		return null
	var channel_count: int = wav_bytes.decode_u16(22)
	var sample_rate: int = wav_bytes.decode_u32(24)
	var bits_per_sample: int = wav_bytes.decode_u16(34)
	var pcm_data: PackedByteArray = wav_bytes.slice(44)
	var stream: AudioStreamWAV = AudioStreamWAV.new()
	stream.mix_rate = sample_rate
	stream.stereo = channel_count > 1
	stream.format = AudioStreamWAV.FORMAT_16_BITS if bits_per_sample >= 16 else AudioStreamWAV.FORMAT_8_BITS
	stream.data = pcm_data
	stream.loop_mode = AudioStreamWAV.LOOP_FORWARD
	stream.loop_begin = 0
	stream.loop_end = pcm_data.size()
	return stream
