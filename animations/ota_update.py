from manim import (
    Tex,
    Scene,
    VGroup,
    Rectangle,
    Arrow,
    Write,
    FadeIn,
    FadeOut,
    Create,
    Transform,
    UP,
    DOWN,
    LEFT,
    RIGHT,
    ORANGE,
    BLUE,
    GREEN,
    RED,
    YELLOW,
    PURPLE,
    GRAY,
    WHITE,
)


class OTAUpdatePipeline(Scene):
    def construct(self):
        # Title
        title = Tex(
            "Beacon OTA Update: Dual-Bank Flash System", font_size=32, color=ORANGE
        ).to_edge(UP)
        self.play(Write(title), run_time=0.5)

        # Part 1: Flash Partition Layout (3s)
        self.show_partition_layout()

        # Part 2: Download and Write (4s)
        self.show_download_write()

        # Part 3: Verification and Activation (3.5s)
        self.show_verification_activation()

        # Part 4: Rollback Safety (2.5s)
        self.show_rollback_safety()

        self.wait(0.5)

    def show_partition_layout(self):
        """Show ESP32-C3 flash partition structure"""
        section_label = (
            Tex("Flash Partition Layout", font_size=20, color=BLUE)
            .to_edge(UP)
            .shift(DOWN * 0.5)
        )
        self.play(Write(section_label), run_time=0.3)

        # Flash partitions
        partitions = [
            ("Bootloader", "0x000000", 0.5, GRAY),
            ("Factory", "0x010000", 1.5, BLUE),
            ("OTA_0", "0x110000", 1.5, GREEN),
            ("OTA_1", "0x210000", 1.5, ORANGE),
            ("OTA Data", "0x310000", 0.5, PURPLE),
        ]

        y_start = 1.5
        partition_objects = []

        for name, addr, height, color in partitions:
            box = Rectangle(
                width=4, height=height, color=color, stroke_width=2, fill_opacity=0.2
            )
            box.shift(DOWN * y_start)

            name_text = Tex(name, font_size=16, color=WHITE).move_to(box.get_center())
            addr_text = Tex(addr, font_size=10, color=GRAY).next_to(
                box, LEFT, buff=0.15
            )

            partition_objects.append((box, name_text, addr_text))
            y_start += height + 0.1

        # Animate partitions
        for box, name, addr in partition_objects:
            self.play(FadeIn(box), Write(name), Write(addr), run_time=0.4)

        self.wait(0.5)

        # Highlight dual banks
        ota0_highlight = partition_objects[2][0].copy().set_stroke(YELLOW, width=4)
        ota1_highlight = partition_objects[3][0].copy().set_stroke(YELLOW, width=4)

        dual_bank_note = Tex(
            "Dual-Bank: OTA\\_0 $\\leftrightarrow$ OTA\\_1", font_size=16, color=YELLOW
        ).next_to(partition_objects[3][0], RIGHT, buff=0.3)

        self.play(
            Create(ota0_highlight),
            Create(ota1_highlight),
            Write(dual_bank_note),
            run_time=0.7,
        )

        self.wait(0.3)

        # Store for next scene
        self.section_label = section_label
        self.partition_objects = partition_objects
        self.ota0_highlight = ota0_highlight
        self.ota1_highlight = ota1_highlight
        self.dual_bank_note = dual_bank_note

    def show_download_write(self):
        """Show firmware download and flash write process"""
        self.play(
            Transform(
                self.section_label,
                Tex("Firmware Download \\& Flash Write", font_size=20, color=GREEN)
                .to_edge(UP)
                .shift(DOWN * 0.5),
            ),
            FadeOut(self.ota0_highlight),
            FadeOut(self.ota1_highlight),
            FadeOut(self.dual_bank_note),
            run_time=0.4,
        )

        # Show current firmware running from OTA_0
        running_label = (
            Tex("Running from OTA\\_0", font_size=12, color=GREEN)
            .next_to(self.partition_objects[2][0], RIGHT, buff=0.2)
            .shift(UP * 0.2)
        )
        self.play(Write(running_label), run_time=0.4)

        # Download process
        download_box = Rectangle(
            width=2.5, height=1, color=BLUE, stroke_width=2, fill_opacity=0.15
        ).shift(LEFT * 4.5 + UP * 0.5)

        download_text = (
            VGroup(
                Tex("WiFi Download", font_size=14, color=BLUE),
                Tex("Firmware v1.2.3", font_size=11, color=WHITE),
                Tex("1MB in 4KB chunks", font_size=9, color=GRAY),
            )
            .arrange(DOWN, buff=0.1)
            .move_to(download_box.get_center())
        )

        self.play(FadeIn(download_box), Write(download_text), run_time=0.6)

        # Show writing to OTA_1
        write_arrow = Arrow(
            download_box.get_right(),
            self.partition_objects[3][0].get_left(),
            color=BLUE,
            stroke_width=3,
        )
        write_label = Tex("Write to OTA\\_1", font_size=12, color=BLUE).next_to(
            write_arrow, UP, buff=0.1
        )

        self.play(Create(write_arrow), Write(write_label), run_time=0.6)

        # Animate progress
        progress_bar = Rectangle(
            width=0.1, height=1.4, color=GREEN, fill_opacity=0.6
        ).align_to(self.partition_objects[3][0], LEFT)

        for i in range(5):
            progress_bar.generate_target()
            progress_bar.target.set_width((i + 1) * 0.8)
            self.play(progress_bar.animate.set_width((i + 1) * 0.8), run_time=0.3)

        self.wait(0.5)

        # Clean up
        self.download_box = download_box
        self.download_text = download_text
        self.write_arrow = write_arrow
        self.write_label = write_label
        self.progress_bar = progress_bar
        self.running_label = running_label

    def show_verification_activation(self):
        """Show checksum verification and partition activation"""
        self.play(
            Transform(
                self.section_label,
                Tex("Verify \\& Activate", font_size=20, color=PURPLE)
                .to_edge(UP)
                .shift(DOWN * 0.5),
            ),
            FadeOut(self.download_box),
            FadeOut(self.download_text),
            FadeOut(self.write_arrow),
            FadeOut(self.write_label),
            FadeOut(self.progress_bar),
            run_time=0.4,
        )

        # Checksum verification
        checksum_box = Rectangle(
            width=2.5, height=0.8, color=YELLOW, stroke_width=2, fill_opacity=0.15
        ).shift(RIGHT * 4 + DOWN * 0.5)

        checksum_text = (
            VGroup(
                Tex("SHA-256 Verify", font_size=14, color=YELLOW),
                Tex("Checksum Match", font_size=11, color=GREEN),
            )
            .arrange(DOWN, buff=0.1)
            .move_to(checksum_box.get_center())
        )

        verify_arrow = Arrow(
            self.partition_objects[3][0].get_right(),
            checksum_box.get_left(),
            color=YELLOW,
            stroke_width=2,
        )

        self.play(
            Create(verify_arrow),
            FadeIn(checksum_box),
            Write(checksum_text),
            run_time=0.8,
        )

        self.wait(0.4)

        # Update OTA Data partition
        ota_data_box = self.partition_objects[4][0]
        ota_data_update = (
            Tex("seq\\_1++", font_size=12, color=PURPLE)
            .next_to(ota_data_box, RIGHT, buff=0.2)
            .shift(DOWN * 0.1)
        )

        update_arrow = Arrow(
            checksum_box.get_bottom(),
            ota_data_box.get_right(),
            color=PURPLE,
            stroke_width=2,
        )

        self.play(Create(update_arrow), Write(ota_data_update), run_time=0.6)

        self.wait(0.3)

        # Reboot indication
        reboot_text = Tex("Reboot...", font_size=18, color=RED).shift(DOWN * 3)
        self.play(Write(reboot_text), run_time=0.4)

        self.wait(0.3)

        # After reboot, running from OTA_1
        new_running_label = (
            Tex("Running from OTA\\_1", font_size=12, color=GREEN)
            .next_to(self.partition_objects[3][0], RIGHT, buff=0.2)
            .shift(UP * 0.2)
        )

        self.play(
            FadeOut(self.running_label),
            FadeOut(checksum_box),
            FadeOut(checksum_text),
            FadeOut(verify_arrow),
            FadeOut(update_arrow),
            FadeOut(ota_data_update),
            FadeOut(reboot_text),
            Write(new_running_label),
            run_time=0.7,
        )

        self.new_running_label = new_running_label

    def show_rollback_safety(self):
        """Show automatic rollback on boot failure"""
        self.play(
            Transform(
                self.section_label,
                Tex("Rollback Safety", font_size=20, color=RED)
                .to_edge(UP)
                .shift(DOWN * 0.5),
            ),
            run_time=0.3,
        )

        # Failure scenario
        failure_text = Tex("If new firmware crashes...", font_size=16, color=RED).shift(
            LEFT * 3.5 + UP * 0.5
        )
        self.play(Write(failure_text), run_time=0.5)

        # Boot count
        boot_attempts = (
            VGroup(
                Tex("Boot attempt 1: Crash", font_size=12, color=RED),
                Tex("Boot attempt 2: Crash", font_size=12, color=RED),
                Tex("Boot attempt 3: Crash", font_size=12, color=RED),
            )
            .arrange(DOWN, aligned_edge=LEFT, buff=0.1)
            .shift(LEFT * 3.5 + DOWN * 0.5)
        )

        for attempt in boot_attempts:
            self.play(Write(attempt), run_time=0.3)

        self.wait(0.3)

        # Bootloader rollback decision
        rollback_decision = Tex(
            "Bootloader: Rollback to OTA\\_0", font_size=14, color=YELLOW
        ).shift(DOWN * 2.5)

        rollback_arrow = Arrow(
            self.partition_objects[3][0].get_left(),
            self.partition_objects[2][0].get_left(),
            color=YELLOW,
            stroke_width=3,
        ).shift(LEFT * 0.5)

        self.play(Write(rollback_decision), Create(rollback_arrow), run_time=0.6)

        self.wait(0.3)

        # Success message
        success_text = Tex(
            "Beacon operational on stable firmware", font_size=16, color=GREEN
        ).to_edge(DOWN)

        self.play(
            FadeOut(failure_text),
            FadeOut(boot_attempts),
            FadeOut(rollback_decision),
            FadeOut(self.new_running_label),
            Write(success_text),
            run_time=0.5,
        )

        # Final highlight on OTA_0
        final_highlight = self.partition_objects[2][0].copy().set_stroke(GREEN, width=4)
        final_label = Tex("Safe \\& Running", font_size=12, color=GREEN).next_to(
            self.partition_objects[2][0], RIGHT, buff=0.2
        )

        self.play(Create(final_highlight), Write(final_label), run_time=0.5)

        self.wait(0.5)
