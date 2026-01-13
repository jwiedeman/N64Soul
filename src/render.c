/**
 * PROJECT NEURON - Render Implementation
 * Visualization system with "Skunkworks Terminal" aesthetic
 */

#include <libdragon.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <math.h>
#include "render.h"

// =============================================================================
// DISPLAY CONTEXT (set by main before rendering)
// =============================================================================

static display_context_t current_disp = 0;

/**
 * Set the current display context for rendering
 * Must be called before any render functions
 */
void render_set_context(display_context_t disp) {
    current_disp = disp;
}

// =============================================================================
// INITIALIZATION
// =============================================================================

void render_init(RenderSettings* settings) {
    settings->show_network = 1;
    settings->show_loss_curve = 1;
    settings->show_histogram = 1;
    settings->show_gradients = 0;
    settings->show_activations = 1;
    settings->scanlines_enabled = 0;  // Disabled by default - too aggressive
    settings->show_metrics = 1;
    settings->histogram_layer = -1;  // All layers
    settings->animation_speed = 1;
    settings->pulse_enabled = 1;
}

// =============================================================================
// COLOR UTILITIES
// =============================================================================

uint32_t color_lerp(uint32_t color1, uint32_t color2, float t) {
    if (t <= 0.0f) return color1;
    if (t >= 1.0f) return color2;

    uint8_t r1 = (color1 >> 24) & 0xFF;
    uint8_t g1 = (color1 >> 16) & 0xFF;
    uint8_t b1 = (color1 >> 8) & 0xFF;
    uint8_t a1 = color1 & 0xFF;

    uint8_t r2 = (color2 >> 24) & 0xFF;
    uint8_t g2 = (color2 >> 16) & 0xFF;
    uint8_t b2 = (color2 >> 8) & 0xFF;
    uint8_t a2 = color2 & 0xFF;

    uint8_t r = (uint8_t)(r1 + (r2 - r1) * t);
    uint8_t g = (uint8_t)(g1 + (g2 - g1) * t);
    uint8_t b = (uint8_t)(b1 + (b2 - b1) * t);
    uint8_t a = (uint8_t)(a1 + (a2 - a1) * t);

    return (r << 24) | (g << 16) | (b << 8) | a;
}

uint32_t color_activation(float activation) {
    // Heat ramp: PHOSPHOR_DIM -> TERMINAL_GREEN -> HOT_WHITE
    activation = activation < 0.0f ? 0.0f : (activation > 1.0f ? 1.0f : activation);

    if (activation < 0.5f) {
        return color_lerp(COLOR_PHOSPHOR_DIM, COLOR_TERMINAL_GREEN, activation * 2.0f);
    } else {
        return color_lerp(COLOR_TERMINAL_GREEN, COLOR_HOT_WHITE, (activation - 0.5f) * 2.0f);
    }
}

uint32_t color_weight(float weight) {
    // Positive = green, Negative = red
    float magnitude = fabsf(weight);
    magnitude = magnitude > 2.0f ? 2.0f : magnitude;  // Clamp to [-2, 2]
    float t = magnitude / 2.0f;

    if (weight >= 0) {
        return color_lerp(COLOR_PHOSPHOR_DIM, COLOR_TERMINAL_GREEN, t);
    } else {
        return color_lerp(COLOR_PHOSPHOR_DIM, COLOR_NEGATIVE_RED, t);
    }
}

// =============================================================================
// PRIMITIVE DRAWING
// =============================================================================

/**
 * Convert our 32-bit RGBA to libdragon color
 */
static uint32_t to_graphics_color(uint32_t rgba) {
    return graphics_make_color(
        (rgba >> 24) & 0xFF,
        (rgba >> 16) & 0xFF,
        (rgba >> 8) & 0xFF,
        rgba & 0xFF
    );
}

void render_rect(int x, int y, int width, int height, uint32_t color) {
    if (!current_disp) return;
    graphics_draw_box(current_disp, x, y, x + width, y + height, to_graphics_color(color));
}

void render_hline(int x1, int y, int length, uint32_t color) {
    if (!current_disp) return;
    graphics_draw_line(current_disp, x1, y, x1 + length, y, to_graphics_color(color));
}

void render_vline(int x, int y1, int length, uint32_t color) {
    if (!current_disp) return;
    graphics_draw_line(current_disp, x, y1, x, y1 + length, to_graphics_color(color));
}

void render_line(int x1, int y1, int x2, int y2, uint32_t color) {
    if (!current_disp) return;
    graphics_draw_line(current_disp, x1, y1, x2, y2, to_graphics_color(color));
}

void render_circle(int cx, int cy, int radius, uint32_t color) {
    if (!current_disp) return;
    // Simple circle approximation using line segments
    uint32_t c = to_graphics_color(color);
    int segments = 16;
    float angle_step = 6.28318530718f / segments;

    for (int i = 0; i < segments; i++) {
        float a1 = i * angle_step;
        float a2 = (i + 1) * angle_step;
        int x1 = cx + (int)(cosf(a1) * radius);
        int y1 = cy + (int)(sinf(a1) * radius);
        int x2 = cx + (int)(cosf(a2) * radius);
        int y2 = cy + (int)(sinf(a2) * radius);
        graphics_draw_line(current_disp, x1, y1, x2, y2, c);
    }
}

void render_circle_filled(int cx, int cy, int radius, uint32_t color) {
    if (!current_disp) return;
    // Fill circle with horizontal lines
    uint32_t c = to_graphics_color(color);
    for (int y = -radius; y <= radius; y++) {
        int half_width = (int)sqrtf((float)(radius * radius - y * y));
        graphics_draw_line(current_disp, cx - half_width, cy + y,
                          cx + half_width, cy + y, c);
    }
}

void render_text(int x, int y, const char* text, uint32_t color) {
    if (!current_disp) return;
    graphics_set_color(to_graphics_color(color), 0);
    graphics_draw_text(current_disp, x, y, text);
}

void render_number(int x, int y, float value, int decimals, uint32_t color) {
    char buf[32];
    switch (decimals) {
        case 0: snprintf(buf, sizeof(buf), "%d", (int)value); break;
        case 1: snprintf(buf, sizeof(buf), "%.1f", value); break;
        case 2: snprintf(buf, sizeof(buf), "%.2f", value); break;
        case 3: snprintf(buf, sizeof(buf), "%.3f", value); break;
        default: snprintf(buf, sizeof(buf), "%.4f", value); break;
    }
    render_text(x, y, buf, color);
}

// =============================================================================
// UI ELEMENTS
// =============================================================================

void render_panel(int x, int y, int width, int height,
                  const char* title, int highlight) {
    uint32_t border_color = highlight ? COLOR_TERMINAL_GREEN : COLOR_PHOSPHOR_DIM;

    // Top border
    render_hline(x, y, width, border_color);
    // Bottom border
    render_hline(x, y + height - 1, width, border_color);
    // Left border
    render_vline(x, y, height, border_color);
    // Right border
    render_vline(x + width - 1, y, height, border_color);

    // Corner brackets if highlighted
    if (highlight) {
        // Top-left
        render_hline(x, y, 4, COLOR_TERMINAL_GREEN);
        render_vline(x, y, 4, COLOR_TERMINAL_GREEN);
        // Top-right
        render_hline(x + width - 4, y, 4, COLOR_TERMINAL_GREEN);
        render_vline(x + width - 1, y, 4, COLOR_TERMINAL_GREEN);
        // Bottom-left
        render_hline(x, y + height - 1, 4, COLOR_TERMINAL_GREEN);
        render_vline(x, y + height - 4, 4, COLOR_TERMINAL_GREEN);
        // Bottom-right
        render_hline(x + width - 4, y + height - 1, 4, COLOR_TERMINAL_GREEN);
        render_vline(x + width - 1, y + height - 4, 4, COLOR_TERMINAL_GREEN);
    }

    // Title
    if (title) {
        int title_x = x + 4;
        int title_y = y - 4;  // Slightly above border
        render_text(title_x, y + 2, title, COLOR_TERMINAL_GREEN);
    }
}

// =============================================================================
// GAME VIEWPORT
// =============================================================================

void render_game_viewport(const PongState* pong,
                          int x, int y, int width, int height) {
    // Draw border
    render_panel(x, y, width, height, "GAME", 0);

    // Playfield background
    int field_x = x + 2;
    int field_y = y + 12;
    int field_w = width - 4;
    int field_h = height - 24;

    // Center line (dashed)
    int center_x = field_x + field_w / 2;
    for (int cy = field_y; cy < field_y + field_h; cy += 8) {
        render_vline(center_x, cy, 4, COLOR_PHOSPHOR_DIM);
    }

    // Scale factors
    float scale_x = (float)field_w / (float)SCREEN_WIDTH;
    float scale_y = (float)field_h / (float)SCREEN_HEIGHT;

    // Draw AI paddle (left)
    int ai_x = field_x + (int)(AI_PADDLE_X * scale_x);
    int ai_y = field_y + (int)((pong->ai_paddle_y - PADDLE_HEIGHT/2) * scale_y);
    int ai_h = (int)(PADDLE_HEIGHT * scale_y);
    render_rect(ai_x, ai_y, (int)(PADDLE_WIDTH * scale_x), ai_h, COLOR_TERMINAL_GREEN);

    // Draw opponent paddle (right)
    int opp_x = field_x + (int)(OPP_PADDLE_X * scale_x);
    int opp_y = field_y + (int)((pong->opp_paddle_y - PADDLE_HEIGHT/2) * scale_y);
    int opp_h = (int)(PADDLE_HEIGHT * scale_y);
    render_rect(opp_x, opp_y, (int)(PADDLE_WIDTH * scale_x), opp_h, COLOR_AMBER_WARN);

    // Draw ball
    int ball_x = field_x + (int)(pong->ball_x * scale_x);
    int ball_y = field_y + (int)(pong->ball_y * scale_y);
    int ball_r = (int)(BALL_SIZE * scale_x / 2);
    if (ball_r < 2) ball_r = 2;
    render_circle_filled(ball_x, ball_y, ball_r, COLOR_HOT_WHITE);

    // Ball trail (velocity indicator)
    if (pong->ball_served) {
        int trail_x = ball_x - (int)(pong->ball_vx * scale_x * 3);
        int trail_y = ball_y - (int)(pong->ball_vy * scale_y * 3);
        render_line(trail_x, trail_y, ball_x, ball_y, COLOR_PHOSPHOR_DIM);
    }

    // Score display
    char score_buf[32];
    snprintf(score_buf, sizeof(score_buf), "AI: %d", pong->ai_score);
    render_text(field_x + 8, y + height - 10, score_buf, COLOR_TERMINAL_GREEN);

    snprintf(score_buf, sizeof(score_buf), "OPP: %d", pong->opp_score);
    render_text(field_x + field_w - 48, y + height - 10, score_buf, COLOR_AMBER_WARN);
}

// =============================================================================
// NETWORK TOPOLOGY
// =============================================================================

void render_network_topology(const NeuralNetwork* nn,
                             int x, int y, int width, int height) {
    render_panel(x, y, width, height, "NETWORK", 0);

    int content_x = x + 4;
    int content_y = y + 14;
    int content_w = width - 8;
    int content_h = height - 18;

    // Calculate layout
    int num_layers = nn->num_layers;
    int layer_spacing = content_w / (num_layers + 1);

    // Find max neurons for vertical spacing
    int max_neurons = 0;
    for (int l = 0; l < num_layers; l++) {
        if (nn->layer_sizes[l] > max_neurons) {
            max_neurons = nn->layer_sizes[l];
        }
    }

    // For larger networks, only show subset of neurons
    int max_display = 12;
    float neuron_skip = 1.0f;
    if (max_neurons > max_display) {
        neuron_skip = (float)max_neurons / (float)max_display;
    }

    int neuron_spacing = content_h / (max_display + 1);
    int neuron_radius = 3;

    // Draw connections (weights) first
    for (int l = 1; l < num_layers; l++) {
        int prev_size = nn->layer_sizes[l - 1];
        int curr_size = nn->layer_sizes[l];

        int prev_display = prev_size > max_display ? max_display : prev_size;
        int curr_display = curr_size > max_display ? max_display : curr_size;

        float prev_skip = (float)prev_size / (float)prev_display;
        float curr_skip = (float)curr_size / (float)curr_display;

        int prev_x = content_x + layer_spacing * l;
        int curr_x = content_x + layer_spacing * (l + 1);

        for (int j = 0; j < curr_display; j++) {
            int actual_j = (int)(j * curr_skip);
            int y2 = content_y + (j + 1) * neuron_spacing;

            for (int i = 0; i < prev_display; i++) {
                int actual_i = (int)(i * prev_skip);
                int y1 = content_y + (i + 1) * neuron_spacing;

                // Get weight
                float weight = nn_get_weight(nn, l, actual_j, actual_i);
                uint32_t color = color_weight(weight);

                // Line thickness based on weight magnitude
                render_line(prev_x, y1, curr_x, y2, color);
            }
        }
    }

    // Draw neurons
    const char* input_labels[] = {"Bx", "By", "Vx", "Vy", "Py", "Oy"};
    const char* output_labels[] = {"UP", "ST", "DN"};

    for (int l = 0; l < num_layers; l++) {
        int layer_size = nn->layer_sizes[l];
        int display_size = layer_size > max_display ? max_display : layer_size;
        float skip = (float)layer_size / (float)display_size;

        int lx = content_x + layer_spacing * (l + 1);

        for (int n = 0; n < display_size; n++) {
            int actual_n = (int)(n * skip);
            int ny = content_y + (n + 1) * neuron_spacing;

            // Get activation
            float activation = nn_get_activation(nn, l, actual_n);
            activation = activation < 0 ? 0 : (activation > 1 ? 1 : activation);

            uint32_t fill_color = color_activation(activation);
            render_circle_filled(lx, ny, neuron_radius, fill_color);
            render_circle(lx, ny, neuron_radius, COLOR_TERMINAL_GREEN);

            // Labels for input/output layers
            if (l == 0 && actual_n < 6) {
                render_text(lx - 16, ny - 3, input_labels[actual_n], COLOR_PHOSPHOR_DIM);
            }
            if (l == num_layers - 1 && actual_n < 3) {
                render_text(lx + 6, ny - 3, output_labels[actual_n], COLOR_PHOSPHOR_DIM);
            }
        }
    }
}

// =============================================================================
// LOSS CURVE
// =============================================================================

void render_loss_curve(const TrainingState* training,
                       int x, int y, int width, int height) {
    render_panel(x, y, width, height, "LOSS", 0);

    int content_x = x + 24;  // Leave room for Y axis labels
    int content_y = y + 12;
    int content_w = width - 32;
    int content_h = height - 24;

    if (training->loss_count < 2) {
        render_text(content_x + content_w/2 - 30, content_y + content_h/2,
                   "NO DATA", COLOR_PHOSPHOR_DIM);
        return;
    }

    // Get range for auto-scaling
    float min_loss, max_loss;
    loss_get_range(training, &min_loss, &max_loss);

    // Draw grid
    for (int i = 0; i <= 4; i++) {
        int gy = content_y + content_h - (i * content_h / 4);
        render_hline(content_x, gy, content_w, COLOR_PHOSPHOR_DIM);

        // Y axis label
        float label_val = min_loss + (max_loss - min_loss) * i / 4.0f;
        char buf[16];
        snprintf(buf, sizeof(buf), "%.1f", label_val);
        render_text(x + 2, gy - 3, buf, COLOR_PHOSPHOR_DIM);
    }

    // Plot loss curve
    int prev_px = -1, prev_py = -1;
    for (int i = 0; i < training->loss_count; i++) {
        float loss = loss_history_get(training, i);

        int px = content_x + (i * content_w / training->loss_count);
        float normalized = (loss - min_loss) / (max_loss - min_loss);
        normalized = normalized < 0 ? 0 : (normalized > 1 ? 1 : normalized);
        int py = content_y + content_h - (int)(normalized * content_h);

        if (prev_px >= 0) {
            render_line(prev_px, prev_py, px, py, COLOR_TERMINAL_GREEN);
        }
        prev_px = px;
        prev_py = py;
    }

    // Current value indicator
    if (prev_px >= 0 && prev_py >= 0) {
        render_circle_filled(prev_px, prev_py, 2, COLOR_HOT_WHITE);
    }

    // Current smoothed loss value
    char loss_buf[32];
    snprintf(loss_buf, sizeof(loss_buf), "%.3f", loss_get_smoothed(training));
    render_text(x + width - 40, y + 2, loss_buf, COLOR_TERMINAL_GREEN);
}

// =============================================================================
// WEIGHT HISTOGRAM
// =============================================================================

void render_histogram(const NeuralNetwork* nn, int layer,
                      int x, int y, int width, int height) {
    render_panel(x, y, width, height, "WEIGHTS", 0);

    int content_x = x + 4;
    int content_y = y + 12;
    int content_w = width - 8;
    int content_h = height - 24;

    // Collect weights into histogram bins
    #define NUM_BINS 24
    int bins[NUM_BINS] = {0};
    float bin_min = -2.0f;
    float bin_max = 2.0f;
    float bin_width = (bin_max - bin_min) / NUM_BINS;

    int total_weights = 0;
    float sum = 0.0f;
    float sum_sq = 0.0f;

    for (int l = 1; l < nn->num_layers; l++) {
        if (layer >= 0 && layer != l) continue;

        int prev_size = nn->layer_sizes[l - 1];
        int curr_size = nn->layer_sizes[l];
        int weight_count = curr_size * prev_size;

        for (int i = 0; i < weight_count; i++) {
            float w = nn->weights[l][i];
            sum += w;
            sum_sq += w * w;
            total_weights++;

            // Bin the weight
            int bin = (int)((w - bin_min) / bin_width);
            if (bin < 0) bin = 0;
            if (bin >= NUM_BINS) bin = NUM_BINS - 1;
            bins[bin]++;
        }
    }

    // Find max bin for scaling
    int max_bin = 1;
    for (int i = 0; i < NUM_BINS; i++) {
        if (bins[i] > max_bin) max_bin = bins[i];
    }

    // Draw histogram bars
    int bar_width = content_w / NUM_BINS;
    for (int i = 0; i < NUM_BINS; i++) {
        int bar_height = (bins[i] * (content_h - 10)) / max_bin;
        int bx = content_x + i * bar_width;
        int by = content_y + content_h - 10 - bar_height;

        // Color based on bin position (negative = red, positive = green)
        float bin_center = bin_min + (i + 0.5f) * bin_width;
        uint32_t bar_color = color_weight(bin_center);

        render_rect(bx, by, bar_width - 1, bar_height, bar_color);
    }

    // X axis labels
    render_text(content_x, content_y + content_h - 8, "-2", COLOR_PHOSPHOR_DIM);
    render_text(content_x + content_w/2 - 4, content_y + content_h - 8, "0", COLOR_PHOSPHOR_DIM);
    render_text(content_x + content_w - 12, content_y + content_h - 8, "+2", COLOR_PHOSPHOR_DIM);

    // Statistics
    if (total_weights > 0) {
        float mean = sum / total_weights;
        float variance = (sum_sq / total_weights) - (mean * mean);
        float std = sqrtf(variance > 0 ? variance : 0);

        char buf[32];
        snprintf(buf, sizeof(buf), "u:%.2f s:%.2f", mean, std);
        render_text(x + 4, y + height - 10, buf, COLOR_PHOSPHOR_DIM);
    }
}

// =============================================================================
// METRICS PANEL
// =============================================================================

void render_metrics(const TrainingState* training,
                    int x, int y, int width, int height) {
    render_panel(x, y, width, height, "METRICS", 0);

    int text_y = y + 14;
    int line_height = 10;

    char buf[32];

    // Win rate
    snprintf(buf, sizeof(buf), "WIN:  %.1f%%", training->win_rate * 100.0f);
    render_text(x + 4, text_y, buf, COLOR_TERMINAL_GREEN);
    text_y += line_height;

    // Episodes
    snprintf(buf, sizeof(buf), "EP:   %d", training->total_episodes);
    render_text(x + 4, text_y, buf, COLOR_TERMINAL_GREEN);
    text_y += line_height;

    // Epsilon
    snprintf(buf, sizeof(buf), "EPS:  %.3f", training->epsilon);
    render_text(x + 4, text_y, buf, COLOR_AMBER_WARN);
    text_y += line_height;

    // Steps
    snprintf(buf, sizeof(buf), "STEP: %dk", training->total_steps / 1000);
    render_text(x + 4, text_y, buf, COLOR_PHOSPHOR_DIM);
}

// =============================================================================
// CONTROLS PANEL
// =============================================================================

void render_controls(int x, int y, int width, int height) {
    render_panel(x, y, width, height, "CTRL", 0);

    int text_y = y + 14;
    int line_height = 9;

    render_text(x + 4, text_y, "A: Speed+", COLOR_PHOSPHOR_DIM);
    text_y += line_height;
    render_text(x + 4, text_y, "B: Speed-", COLOR_PHOSPHOR_DIM);
    text_y += line_height;
    render_text(x + 4, text_y, "Z: Play", COLOR_PHOSPHOR_DIM);
    text_y += line_height;
    render_text(x + 4, text_y, "START:Menu", COLOR_PHOSPHOR_DIM);
}

// =============================================================================
// HEADER BAR
// =============================================================================

void render_header(const TrainingState* training, int speed_multiplier) {
    // Background bar
    render_rect(0, 0, SCREEN_WIDTH, 12, COLOR_PHOSPHOR_DIM);

    char buf[64];
    snprintf(buf, sizeof(buf), "NEURON EP:%05d e:%.2f n:%.4f [%dx]",
             training->total_episodes,
             training->epsilon,
             training->learning_rate,
             speed_multiplier);
    render_text(4, 2, buf, COLOR_TERMINAL_GREEN);

    // Training indicator
    render_text(SCREEN_WIDTH - 60, 2, "[TRAIN]", COLOR_AMBER_WARN);
}

// =============================================================================
// SCANLINE EFFECT
// =============================================================================

void render_scanlines(void) {
    // Every other line, draw a semi-transparent dark line
    // This is expensive so we do it sparingly
    for (int y = 0; y < SCREEN_HEIGHT; y += 2) {
        render_hline(0, y, SCREEN_WIDTH, 0x00000040);  // Very transparent black
    }
}

// =============================================================================
// FLASH EFFECT
// =============================================================================

void render_flash(int x, int y, int intensity) {
    if (intensity <= 0) return;

    // Small bright circle that fades out
    uint8_t alpha = (uint8_t)(intensity > 255 ? 255 : intensity);
    uint32_t color = (0xFF << 24) | (0xFF << 16) | (0xFF << 8) | alpha;
    render_circle_filled(x, y, 2, color);
}

// =============================================================================
// MAIN FRAME RENDER
// =============================================================================

void render_frame(const PongState* pong,
                  const NeuralNetwork* nn,
                  const TrainingState* training,
                  const RenderSettings* settings) {
    // Display context should be set via render_set_context before calling
    if (!current_disp) return;

    // Header bar
    render_header(training, 1);  // TODO: pass actual speed

    // Layout constants
    int header_h = 14;
    int game_w = 140;
    int game_h = 100;
    int network_w = SCREEN_WIDTH - game_w - 8;
    int network_h = 100;
    int loss_h = 50;
    int bottom_h = 40;

    // Game viewport (left side)
    render_game_viewport(pong, 2, header_h + 2, game_w, game_h);

    // Network topology (right side)
    if (settings->show_network) {
        render_network_topology(nn, game_w + 4, header_h + 2, network_w, network_h);
    }

    // Loss curve (full width, below game/network)
    if (settings->show_loss_curve) {
        render_loss_curve(training, 2, header_h + game_h + 4, SCREEN_WIDTH - 4, loss_h);
    }

    // Bottom row: metrics, histogram, controls
    int bottom_y = header_h + game_h + loss_h + 6;
    int panel_w = (SCREEN_WIDTH - 8) / 3;

    if (settings->show_metrics) {
        render_metrics(training, 2, bottom_y, panel_w - 2, bottom_h);
    }

    if (settings->show_histogram) {
        render_histogram(nn, settings->histogram_layer,
                        panel_w + 2, bottom_y, panel_w - 2, bottom_h);
    }

    render_controls(panel_w * 2 + 2, bottom_y, panel_w - 2, bottom_h);

    // Note: scanlines are applied by main.c after all rendering
    // Note: display_show is called by main.c
}
