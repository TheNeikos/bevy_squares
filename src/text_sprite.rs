use bevy::prelude::*;
use bevy::render::{
    draw::{DrawContext, Drawable},
    renderer::{AssetRenderResourceBindings, RenderResourceBindings},
};
use bevy::text::{DrawableText, FontAtlasSet};

#[derive(Default)]
pub struct TextSprite {
    pub text: String,
    pub text_style: TextStyle,
    pub font: Handle<Font>,
}

#[derive(Default)]
pub struct TextSpriteSize(Vec2);

pub fn update_text_sprites(
    mut textures: ResMut<Assets<Texture>>,
    fonts: Res<Assets<Font>>,
    mut font_atlas_sets: ResMut<Assets<FontAtlasSet>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut query: Query<(Changed<TextSprite>, &mut TextSpriteSize)>,
) {
    for (text_sprite, mut sprite_size) in &mut query.iter() {
        let font_atlases = font_atlas_sets
            .get_or_insert_with(Handle::from_id(text_sprite.font.id), || {
                FontAtlasSet::new(text_sprite.font)
            });
        let width = font_atlases.add_glyphs_to_atlas(
            &fonts,
            &mut texture_atlases,
            &mut textures,
            text_sprite.text_style.font_size,
            &text_sprite.text,
        );

        sprite_size.0 = Vec2::new(width, text_sprite.text_style.font_size);
    }
}

pub fn draw_text_sprites(
    mut draw_context: DrawContext,
    fonts: Res<Assets<Font>>,
    msaa: Res<Msaa>,
    font_atlas_sets: Res<Assets<FontAtlasSet>>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut render_resource_bindings: ResMut<RenderResourceBindings>,
    mut asset_render_resource_bindings: ResMut<AssetRenderResourceBindings>,
    mut query: Query<(&mut Draw, &TextSprite, &TextSpriteSize, &Translation)>,
) {
    for (mut draw, text_sprite, text_sprite_size, translate) in &mut query.iter() {
        let position = translate.0 - (text_sprite_size.0 / 2.).extend(0.);

        let mut drawable_text = DrawableText {
            font: fonts.get(&text_sprite.font).unwrap(),
            font_atlas_set: font_atlas_sets
                .get(&text_sprite.font.as_handle::<FontAtlasSet>())
                .unwrap(),
            texture_atlases: &texture_atlases,
            render_resource_bindings: &mut render_resource_bindings,
            asset_render_resource_bindings: &mut asset_render_resource_bindings,
            position,
            msaa: &msaa,
            style: &text_sprite.text_style,
            text: &text_sprite.text,
            container_size: text_sprite_size.0,
        };
        drawable_text.draw(&mut draw, &mut draw_context).unwrap();
    }
}
