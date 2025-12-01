fn main() {
    glazer::run(
        tea_sir::memory(),
        tea_sir::MAX_WIDTH,
        tea_sir::MAX_HEIGHT,
        tea_sir::handle_input,
        tea_sir::update_and_render,
        glazer::debug_target(),
    );
}
