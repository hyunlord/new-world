// Opaque person pixels mark their own coverage without a second body draw.
shader_type spatial;
render_mode diffuse_lambert, specular_disabled;
stencil_mode write, 27;

#include "res://shaders/world3d/vat_surface.gdshaderinc"
