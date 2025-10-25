from manim import *


class BeaconUnlockLogic(Scene):
    def construct(self):
        # Title (0.5s)
        title = Tex("Beacon Unlock Logic", font_size=36, color=RED).to_edge(UP)
        self.play(Write(title), run_time=0.5)

        # Setup (0.5s)
        self.setup_components()

        # Scenario: Human sensor triggered (3s)
        self.show_unlock_sequence()

        self.wait(0.5)

    def setup_components(self):
        """Show beacon hardware components"""
        # Beacon box
        beacon_box = Rectangle(
            width=3, height=4, color=RED, stroke_width=3, fill_opacity=0.1
        )
        beacon_label = Tex("ESP32-C3", font_size=20, color=RED).next_to(
            beacon_box.get_top(), DOWN, buff=0.2
        )

        # Components inside beacon
        human_sensor = Rectangle(
            width=1.2, height=0.6, color=BLUE, stroke_width=2, fill_opacity=0.2
        )
        human_label = Tex("Human\\\\Sensor", font_size=12, color=BLUE)
        human_group = VGroup(
            human_sensor, human_label.move_to(human_sensor.get_center())
        )
        human_group.move_to(beacon_box.get_top() + DOWN * 1)

        relay = Rectangle(
            width=1.2, height=0.6, color=ORANGE, stroke_width=2, fill_opacity=0.2
        )
        relay_label = Tex("Relay", font_size=12, color=ORANGE)
        relay_group = VGroup(relay, relay_label.move_to(relay.get_center()))
        relay_group.move_to(beacon_box.get_center())

        timer_label = Tex("Timer: 0s", font_size=14, color=GRAY).move_to(
            beacon_box.get_bottom() + UP * 0.5
        )

        beacon_full = VGroup(
            beacon_box, beacon_label, human_group, relay_group, timer_label
        )
        beacon_full.shift(LEFT * 3)

        # Door/lock
        door = Rectangle(
            width=1.5, height=3, color=GRAY, stroke_width=3, fill_opacity=0.15
        )
        lock_icon = Tex("Locked", font_size=40).move_to(door.get_center())
        door_group = VGroup(door, lock_icon)
        door_group.shift(RIGHT * 3)

        # Connection wire
        wire = DashedLine(
            relay_group.get_right(), door_group.get_left(), color=ORANGE, stroke_width=2
        )

        self.play(FadeIn(beacon_full), FadeIn(door_group), Create(wire), run_time=0.5)

        self.beacon_box = beacon_box
        self.human_sensor = human_sensor
        self.relay = relay
        self.timer_label = timer_label
        self.door_group = door_group
        self.lock_icon = lock_icon
        self.wire = wire

    def show_unlock_sequence(self):
        """Show the unlock sequence with timing"""
        # Stage 1: Human detected (0.5s)
        detection = Tex("Human Detected!", font_size=16, color=YELLOW).next_to(
            self.human_sensor, RIGHT, buff=0.3
        )
        pulse = Circle(radius=0.4, color=BLUE, stroke_width=3).move_to(
            self.human_sensor.get_center()
        )

        self.play(
            self.human_sensor.animate.set_fill(BLUE, opacity=0.6),
            Create(pulse),
            Write(detection),
            run_time=0.3,
        )
        self.play(FadeOut(pulse), run_time=0.2)

        # Stage 2: Relay activates (0.5s)
        relay_signal = Tex("Relay HIGH", font_size=14, color=ORANGE).next_to(
            self.relay, LEFT, buff=0.3
        )

        self.play(
            self.relay.animate.set_fill(ORANGE, opacity=0.7),
            self.wire.animate.set_color(YELLOW).set_stroke(width=4),
            Write(relay_signal),
            run_time=0.3,
        )

        # Stage 3: Door unlocks (0.4s)
        unlock_icon = Tex("Unlocked", font_size=40).move_to(
            self.door_group.get_center()
        )

        self.play(
            Transform(self.lock_icon, unlock_icon),
            self.door_group[0].animate.set_stroke(GREEN, width=4),
            run_time=0.4,
        )

        # Stage 4: Timer countdown (1.2s)
        timers = ["5s", "4s", "3s", "2s", "1s", "0s"]
        labels = ["last\\_trigger", "...", "...", "...", "...", "timeout"]

        for i, (time, label) in enumerate(zip(timers, labels)):
            new_timer = Tex(
                f"Timer: {time}", font_size=14, color=YELLOW if i < 5 else RED
            )
            new_timer.move_to(self.timer_label.get_center())

            status = Tex(label, font_size=11, color=GRAY).next_to(
                new_timer, DOWN, buff=0.1
            )

            self.play(
                Transform(self.timer_label, new_timer),
                FadeIn(status) if i == 0 or i == 5 else FadeOut(status),
                run_time=0.15 if i < 5 else 0.2,
            )

            if i < 5:
                self.remove(status)

        # Stage 5: Close relay (0.5s)
        close_signal = Tex("Relay LOW", font_size=14, color=GRAY).next_to(
            self.relay, LEFT, buff=0.3
        )

        self.play(
            self.relay.animate.set_fill(GRAY, opacity=0.2),
            self.wire.animate.set_color(GRAY).set_stroke(width=2),
            Transform(relay_signal, close_signal),
            self.human_sensor.animate.set_fill(BLUE, opacity=0.2),
            FadeOut(detection),
            run_time=0.4,
        )

        # Stage 6: Door locks (0.3s)
        lock_icon_new = Tex("Locked", font_size=40).move_to(
            self.door_group.get_center()
        )

        self.play(
            Transform(self.lock_icon, lock_icon_new),
            self.door_group[0].animate.set_stroke(GRAY, width=3),
            run_time=0.3,
        )

        # Summary (show final state)
        summary = (
            Tex(
                "Logic: Hold 5s after trigger\\\\Close 5s after relay HIGH",
                font_size=12,
                color=YELLOW,
            )
            .to_edge(DOWN)
            .shift(UP * 0.3)
        )

        self.play(Write(summary), run_time=0.5)
