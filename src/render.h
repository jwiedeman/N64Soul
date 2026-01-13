/**
 * PROJECT NEURON - Render Header
 * Visualization and drawing functions
 */

#ifndef RENDER_H
#define RENDER_H

#include <stdint.h>
#include <libdragon.h>
#include "../include/config.h"
#include "neural_net.h"
#include "training.h"
#include "pong.h"

// =============================================================================
// RENDER SETTINGS
// =============================================================================

typedef struct {
    // Toggles for visualization components
    uint8_t show_network;       // Network topology view
    uint8_t show_loss_curve;    // Loss curve graph
    uint8_t show_histogram;     // Weight distribution histogram
    uint8_t show_gradients;     // Gradient flow animation
    uint8_t show_activations;   // Activation flow animation

    // Display options
    uint8_t scanlines_enabled;  // CRT scanline effect
    uint8_t show_metrics;       // Stats panel

    // Histogram layer selection (-1 = all layers)
    int histogram_layer;

    // Animation settings
    int animation_speed;        // Weight transition speed
    int pulse_enabled;          // Activation pulse animation

} RenderSettings;

// =============================================================================
// INITIALIZATION
// =============================================================================

/**
 * Initialize render settings with defaults
 * @param settings Pointer to render settings
 */
void render_init(RenderSettings* settings);

/**
 * Set the display context for rendering
 * Must be called before any render functions each frame
 * @param disp Display context from display_lock()
 */
void render_set_context(display_context_t disp);

// =============================================================================
// MAIN RENDER FUNCTION
// =============================================================================

/**
 * Render complete frame
 * @param pong Pointer to pong state
 * @param nn Pointer to neural network
 * @param training Pointer to training state
 * @param settings Pointer to render settings
 */
void render_frame(const PongState* pong,
                  const NeuralNetwork* nn,
                  const TrainingState* training,
                  const RenderSettings* settings);

// =============================================================================
// COMPONENT RENDERERS
// =============================================================================

/**
 * Render status header bar
 * @param training Pointer to training state
 * @param speed_multiplier Current simulation speed
 */
void render_header(const TrainingState* training, int speed_multiplier);

/**
 * Render pong game viewport
 * @param pong Pointer to pong state
 * @param x Panel X position
 * @param y Panel Y position
 * @param width Panel width
 * @param height Panel height
 */
void render_game_viewport(const PongState* pong,
                          int x, int y, int width, int height);

/**
 * Render neural network topology
 * @param nn Pointer to neural network
 * @param x Panel X position
 * @param y Panel Y position
 * @param width Panel width
 * @param height Panel height
 */
void render_network_topology(const NeuralNetwork* nn,
                             int x, int y, int width, int height);

/**
 * Render loss curve graph
 * @param training Pointer to training state
 * @param x Panel X position
 * @param y Panel Y position
 * @param width Panel width
 * @param height Panel height
 */
void render_loss_curve(const TrainingState* training,
                       int x, int y, int width, int height);

/**
 * Render weight histogram
 * @param nn Pointer to neural network
 * @param layer Layer to display (-1 for all)
 * @param x Panel X position
 * @param y Panel Y position
 * @param width Panel width
 * @param height Panel height
 */
void render_histogram(const NeuralNetwork* nn, int layer,
                      int x, int y, int width, int height);

/**
 * Render metrics panel
 * @param training Pointer to training state
 * @param x Panel X position
 * @param y Panel Y position
 * @param width Panel width
 * @param height Panel height
 */
void render_metrics(const TrainingState* training,
                    int x, int y, int width, int height);

/**
 * Render controls help panel
 * @param x Panel X position
 * @param y Panel Y position
 * @param width Panel width
 * @param height Panel height
 */
void render_controls(int x, int y, int width, int height);

// =============================================================================
// UI PRIMITIVES
// =============================================================================

/**
 * Draw panel with border
 * @param x Panel X position
 * @param y Panel Y position
 * @param width Panel width
 * @param height Panel height
 * @param title Panel title (NULL for no title)
 * @param highlight Use highlight border style
 */
void render_panel(int x, int y, int width, int height,
                  const char* title, int highlight);

/**
 * Draw text with terminal font
 * @param x X position
 * @param y Y position
 * @param text Text to draw
 * @param color RGBA color
 */
void render_text(int x, int y, const char* text, uint32_t color);

/**
 * Draw formatted number
 * @param x X position
 * @param y Y position
 * @param value Numeric value
 * @param decimals Number of decimal places
 * @param color RGBA color
 */
void render_number(int x, int y, float value, int decimals, uint32_t color);

/**
 * Draw horizontal line
 * @param x1 Start X
 * @param y Y position
 * @param length Line length
 * @param color RGBA color
 */
void render_hline(int x1, int y, int length, uint32_t color);

/**
 * Draw vertical line
 * @param x X position
 * @param y1 Start Y
 * @param length Line length
 * @param color RGBA color
 */
void render_vline(int x, int y1, int length, uint32_t color);

/**
 * Draw line between two points
 * @param x1 Start X
 * @param y1 Start Y
 * @param x2 End X
 * @param y2 End Y
 * @param color RGBA color
 */
void render_line(int x1, int y1, int x2, int y2, uint32_t color);

/**
 * Draw filled rectangle
 * @param x X position
 * @param y Y position
 * @param width Rectangle width
 * @param height Rectangle height
 * @param color RGBA color
 */
void render_rect(int x, int y, int width, int height, uint32_t color);

/**
 * Draw circle outline
 * @param cx Center X
 * @param cy Center Y
 * @param radius Circle radius
 * @param color RGBA color
 */
void render_circle(int cx, int cy, int radius, uint32_t color);

/**
 * Draw filled circle
 * @param cx Center X
 * @param cy Center Y
 * @param radius Circle radius
 * @param color RGBA color
 */
void render_circle_filled(int cx, int cy, int radius, uint32_t color);

// =============================================================================
// EFFECTS
// =============================================================================

/**
 * Apply scanline effect to framebuffer
 */
void render_scanlines(void);

/**
 * Draw weight flash effect
 * @param x X position
 * @param y Y position
 * @param intensity Flash intensity (0-255)
 */
void render_flash(int x, int y, int intensity);

// =============================================================================
// COLOR UTILITIES
// =============================================================================

/**
 * Interpolate between two colors
 * @param color1 First color
 * @param color2 Second color
 * @param t Interpolation factor (0.0 to 1.0)
 * @return Interpolated color
 */
uint32_t color_lerp(uint32_t color1, uint32_t color2, float t);

/**
 * Get activation heat color
 * @param activation Activation value (0.0 to 1.0)
 * @return Color on heat ramp
 */
uint32_t color_activation(float activation);

/**
 * Get weight color (positive = green, negative = red)
 * @param weight Weight value
 * @return Color representing weight
 */
uint32_t color_weight(float weight);

#endif // RENDER_H
