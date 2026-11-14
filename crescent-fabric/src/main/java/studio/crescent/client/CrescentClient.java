package studio.crescent.client;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import net.minecraft.text.Text;
import org.lwjgl.glfw.GLFW;

public final class CrescentClient implements ClientModInitializer {
    private static KeyBinding openMenu;
    private static boolean showFps = true;
    private static boolean showKeystrokes = true;

    @Override
    public void onInitializeClient() {
        openMenu = KeyBindingHelper.registerKeyBinding(new KeyBinding(
            "key.crescent.open_menu",
            InputUtil.Type.KEYSYM,
            GLFW.GLFW_KEY_RIGHT_SHIFT,
            "category.crescent"
        ));

        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            while (openMenu.wasPressed()) {
                client.setScreen(new CrescentMenuScreen());
            }
        });

        HudRenderCallback.EVENT.register((drawContext, tickCounter) -> {
            MinecraftClient client = MinecraftClient.getInstance();
            if (client.player == null || client.options.hudHidden) return;

            int y = 8;
            if (showFps) {
                drawContext.drawTextWithShadow(client.textRenderer,
                    Text.literal("Crescent  |  " + client.getCurrentFps() + " FPS"),
                    8, y, 0xFFE4E8F0);
                y += 12;
            }

            if (showKeystrokes) {
                drawKeystrokes(drawContext, client, 8, y);
            }
        });
    }

    private static void drawKeystrokes(DrawContext context, MinecraftClient client, int x, int y) {
        boolean forward = client.options.forwardKey.isPressed();
        boolean left = client.options.leftKey.isPressed();
        boolean back = client.options.backKey.isPressed();
        boolean right = client.options.rightKey.isPressed();
        drawKey(context, client, "W", x + 22, y, forward);
        drawKey(context, client, "A", x, y + 22, left);
        drawKey(context, client, "S", x + 22, y + 22, back);
        drawKey(context, client, "D", x + 44, y + 22, right);
    }

    private static void drawKey(DrawContext context, MinecraftClient client, String label,
                                int x, int y, boolean pressed) {
        int background = pressed ? 0xCCCAD5E3 : 0xAA171B23;
        int foreground = pressed ? 0xFF11151C : 0xFFF0F2F7;
        context.fill(x, y, x + 20, y + 20, background);
        context.drawCenteredTextWithShadow(client.textRenderer, Text.literal(label),
            x + 10, y + 6, foreground);
    }

    static boolean isShowFps() { return showFps; }
    static void setShowFps(boolean value) { showFps = value; }
    static boolean isShowKeystrokes() { return showKeystrokes; }
    static void setShowKeystrokes(boolean value) { showKeystrokes = value; }

    private static final class CrescentMenuScreen extends Screen {
        protected CrescentMenuScreen() {
            super(Text.literal("Crescent Client"));
        }

        @Override
        protected void init() {
            int center = this.width / 2;
            int top = this.height / 2 - 76;

            addDrawableChild(net.minecraft.client.gui.widget.ButtonWidget.builder(
                Text.literal("FPS Counter: " + (showFps ? "ON" : "OFF")),
                button -> {
                    showFps = !showFps;
                    button.setMessage(Text.literal("FPS Counter: " + (showFps ? "ON" : "OFF")));
                }).dimensions(center - 105, top, 210, 20).build());

            addDrawableChild(net.minecraft.client.gui.widget.ButtonWidget.builder(
                Text.literal("Keystrokes: " + (showKeystrokes ? "ON" : "OFF")),
                button -> {
                    showKeystrokes = !showKeystrokes;
                    button.setMessage(Text.literal("Keystrokes: " + (showKeystrokes ? "ON" : "OFF")));
                }).dimensions(center - 105, top + 26, 210, 20).build());

            addDrawableChild(net.minecraft.client.gui.widget.ButtonWidget.builder(
                Text.literal("Done"),
                button -> close()).dimensions(center - 105, top + 56, 210, 20).build());
        }

        @Override
        public void render(DrawContext context, int mouseX, int mouseY, float delta) {
            renderBackground(context, mouseX, mouseY, delta);
            context.fill(width / 2 - 132, height / 2 - 104,
                width / 2 + 132, height / 2 + 96, 0xE910131A);
            context.drawCenteredTextWithShadow(textRenderer,
                Text.literal("CRESCENT CLIENT"), width / 2, height / 2 - 91, 0xFFE7EBF3);
            super.render(context, mouseX, mouseY, delta);
        }

        @Override
        public boolean shouldPause() {
            return false;
        }
    }
}
