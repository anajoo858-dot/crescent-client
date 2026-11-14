package studio.crescent.client;

import net.fabricmc.api.ClientModInitializer;

public final class CrescentClient implements ClientModInitializer {
    @Override
    public void onInitializeClient() {
        System.out.println("[Crescent] Crescent Client for Minecraft 26.3 initialized.");
    }
}
