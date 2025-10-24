from manim import *
import numpy as np

class LocalizationVisualization(Scene):
    def construct(self):
        # Title (0.5s)
        title = Text("Navign BLE Localization Pipeline").scale(0.6).to_edge(UP)
        self.play(Write(title), run_time=0.5)
        
        # Part 1: BLE Scanning (3s)
        self.ble_scanning()
        
        # Part 2: Area Selection (3s)  
        self.area_selection()
        
        # Part 3: Position Calculation (8s)
        self.position_calculation()
        
        self.wait(0.5)
    
    def ble_scanning(self):
        """BLE beacon scanning"""
        step_label = Text("1. BLE Scan (5s)", font_size=20, color=BLUE).to_edge(UP).shift(DOWN*0.5)
        self.play(Write(step_label), run_time=0.3)
        
        # Phone
        phone = Rectangle(width=0.6, height=1.0, color=BLUE, stroke_width=3, fill_opacity=0.2)
        phone_icon = Text("📱", font_size=32).move_to(phone.get_center())
        phone_group = VGroup(phone, phone_icon)
        
        self.play(FadeIn(phone_group), run_time=0.4)
        
        # Beacons appear with RSSI
        beacons = []
        beacon_data = [
            (LEFT*2.5 + UP*1.5, -45, "B1"),
            (RIGHT*2.5 + UP*1.5, -65, "B2"),
            (LEFT*2.5 + DOWN*1.5, -72, "B3"),
            (RIGHT*2.5 + DOWN*1.5, -85, "B4"),
            (UP*2, -155, "B5"),
        ]
        
        for pos, rssi, label in beacon_data:
            beacon = Circle(radius=0.15, color=GREEN, fill_opacity=0.8).move_to(pos)
            rssi_text = Text(f"{rssi}dBm", font_size=12, color=WHITE).next_to(beacon, UP, buff=0.1)
            label_text = Text(label, font_size=10, color=GRAY).next_to(beacon, DOWN, buff=0.05)
            
            # Signal waves
            wave = Circle(radius=0.3, color=GREEN, stroke_width=1.5, fill_opacity=0).move_to(pos)
            
            beacons.append({
                "beacon": beacon,
                "rssi": rssi,
                "rssi_text": rssi_text,
                "label": label_text,
                "wave": wave,
                "pos": pos
            })
        
        # Animate beacon discovery
        for b in beacons:
            self.play(
                FadeIn(b["beacon"]),
                Write(b["rssi_text"]),
                Write(b["label"]),
                Create(b["wave"]),
                run_time=0.3
            )
        
        self.wait(0.3)
        
        # Store for next step
        self.phone_group = phone_group
        self.beacons = beacons
        self.step_label = step_label
    
    def area_selection(self):
        """Group by area and select best area"""
        self.play(Transform(self.step_label, 
            Text("2. Area Selection", font_size=20, color=GREEN).to_edge(UP).shift(DOWN*0.5)),
            run_time=0.3)
        
        # Draw area boundaries
        area_a = Rectangle(width=3, height=3.5, color=BLUE, stroke_width=2, fill_opacity=0.05).shift(LEFT*2)
        area_b = Rectangle(width=3, height=3.5, color=ORANGE, stroke_width=2, fill_opacity=0.05).shift(RIGHT*2)
        
        area_a_label = Text("Area A (3 beacons)", font_size=14, color=BLUE).next_to(area_a, UP, buff=0.1)
        area_b_label = Text("Area B (2 beacons)", font_size=14, color=ORANGE).next_to(area_b, UP, buff=0.1)
        
        self.play(
            Create(area_a),
            Create(area_b),
            Write(area_a_label),
            Write(area_b_label),
            run_time=0.8
        )
        
        # Highlight effective beacons (RSSI >= -160)
        threshold_line = DashedLine(LEFT*3.5 + DOWN*2.5, RIGHT*3.5 + DOWN*2.5, color=RED, stroke_width=2)
        threshold_label = Text("RSSI threshold: -160 dBm", font_size=12, color=RED).next_to(threshold_line, DOWN, buff=0.1)
        
        # Show selection
        selection_box = SurroundingRectangle(area_a, color=YELLOW, stroke_width=4, buff=0.1)
        selection_text = Text("✓ Area A selected", font_size=16, color=YELLOW).to_edge(DOWN)
        
        self.play(
            Create(selection_box),
            Write(selection_text),
            run_time=0.8
        )
        
        self.wait(0.4)
        
        # Clean up for next step
        self.play(
            FadeOut(area_a), FadeOut(area_b),
            FadeOut(area_a_label), FadeOut(area_b_label),
            FadeOut(selection_box), FadeOut(selection_text),
            FadeOut(VGroup(*[b["wave"] for b in self.beacons[1:]])),  # Keep only Area A beacons
            FadeOut(self.beacons[1]["beacon"]), FadeOut(self.beacons[1]["rssi_text"]), FadeOut(self.beacons[1]["label"]),
            FadeOut(self.beacons[3]["beacon"]), FadeOut(self.beacons[3]["rssi_text"]), FadeOut(self.beacons[3]["label"]),
            run_time=0.4
        )
        
        # Keep only Area A beacons
        self.active_beacons = [self.beacons[0], self.beacons[2], self.beacons[4]]
    
    def position_calculation(self):
        """Calculate position using RSSI"""
        self.play(Transform(self.step_label,
            Text("3. Position Calculation", font_size=20, color=PURPLE).to_edge(UP).shift(DOWN*0.5)),
            run_time=0.3)
        
        # Show RSSI thresholds
        threshold_box = VGroup(
            Text("RSSI Thresholds:", font_size=14, color=YELLOW),
            Text("≥ -60 dBm: Use strongest", font_size=12, color=GREEN),
            Text("-60 to -160: Weighted centroid", font_size=12, color=BLUE),
            Text("< -160: Ignore", font_size=12, color=RED)
        ).arrange(DOWN, aligned_edge=LEFT, buff=0.1).to_corner(UR).shift(LEFT*0.2 + DOWN*0.5)
        
        self.play(FadeIn(threshold_box), run_time=0.5)
        
        # Scenario: Weighted centroid (all beacons -60 to -160)
        calc_label = Text("Weighted Centroid Method", font_size=16, color=BLUE).to_edge(DOWN).shift(UP*2)
        self.play(Write(calc_label), run_time=0.4)
        
        # Show formula
        formula = MathTex(
            r"\text{position} = \frac{\sum (beacon_i \times w_i)}{\sum w_i}",
            r"\text{ where } w_i = \frac{1}{distance_i}",
            font_size=32
        ).scale(0.6).next_to(calc_label, DOWN, buff=0.2)
        self.play(Write(formula), run_time=0.6)
        
        # Calculate and show weights
        weights = []
        total_weight = 0
        weighted_x = 0
        weighted_y = 0
        
        for b in self.active_beacons:
            # RSSI to distance: 10^((TxPower - RSSI) / (10 * n))
            tx_power = -59
            n = 2.0
            rssi = b["rssi"]
            distance = 10 ** ((tx_power - rssi) / (10 * n))
            weight = 1.0 / distance if distance > 0 else 0
            
            weights.append(weight)
            total_weight += weight
            
            pos = b["pos"]
            weighted_x += pos[0] * weight
            weighted_y += pos[1] * weight
            
            # Show weight line
            weight_line = Line(
                self.phone_group.get_center(),
                b["beacon"].get_center(),
                color=BLUE,
                stroke_width=2 + weight * 3
            )
            weight_text = Text(f"w={weight:.2f}", font_size=10, color=BLUE).move_to(
                (self.phone_group.get_center() + b["beacon"].get_center()) / 2
            ).shift(UP*0.2)
            
            self.play(Create(weight_line), Write(weight_text), run_time=0.4)
        
        # Calculate final position
        final_x = weighted_x / total_weight
        final_y = weighted_y / total_weight
        final_pos = np.array([final_x, final_y, 0])
        
        # Animate phone moving to calculated position
        result_marker = Dot(final_pos, color=YELLOW, radius=0.15)
        result_circle = Circle(radius=0.3, color=YELLOW, stroke_width=3).move_to(final_pos)
        coords_text = Text(f"({final_x:.1f}, {final_y:.1f})", font_size=14, color=YELLOW).next_to(result_marker, DOWN, buff=0.3)
        
        self.play(
            self.phone_group.animate.move_to(final_pos),
            FadeIn(result_marker),
            Create(result_circle),
            run_time=1.0
        )
        
        self.play(Write(coords_text), run_time=0.4)
        
        # Success message
        success = Text("✓ Position Calculated", font_size=18, color=GREEN).to_edge(DOWN)
        self.play(Write(success), run_time=0.4)
        
        self.wait(0.5)
