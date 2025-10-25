from manim import *


class RobotArchitecture(Scene):
    def construct(self):
        # Define layout boundaries
        upper_boundary = 2.5
        lower_boundary = -1.5
        separator_line = Line(LEFT * 7, RIGHT * 7, color=GRAY).shift(DOWN * 1)

        # Title with LaTeX
        title = Tex(
            r"\textbf{Dual-Purpose Robot Dog Architecture}", font_size=40
        ).to_edge(UP)
        self.play(Write(title))
        self.wait(0.5)

        # Draw separator
        self.play(Create(separator_line))

        # Layer labels with LaTeX
        upper_label = Tex(
            r"\textsc{Upper Layer} \\ \texttt{(Raspberry Pi/Jetson)}",
            font_size=28,
            color=BLUE,
        ).move_to(UP * 3.2)
        lower_label = Tex(
            r"\textsc{Lower Layer} \\ \texttt{(STM32 + Embassy Rust)}",
            font_size=28,
            color=GREEN,
        ).move_to(DOWN * 3.2)
        self.play(Write(upper_label), Write(lower_label))
        self.wait(0.5)

        # ===== UPPER LAYER =====
        # ROS2 core in center
        ros_core = Circle(radius=0.6, color=BLUE, fill_opacity=0.3).shift(UP * 0.5)
        ros_text = Tex(
            r"\texttt{ROS2} \\ \texttt{Core}", font_size=24, color=WHITE
        ).move_to(ros_core.get_center())
        ros_group = VGroup(ros_core, ros_text)

        self.play(FadeIn(ros_core), Write(ros_text))
        self.wait(0.5)

        # Six subsystems arranged in circle around ROS
        subsystems = [
            (r"\textsc{Vision}", UP * 2.2, YELLOW),
            (r"\textsc{Audio}", UP * 1.8 + RIGHT * 2, ORANGE),
            (r"\textsc{Bluetooth}", DOWN * 0.2 + RIGHT * 2.5, PURPLE),
            (r"\textsc{Navign}", DOWN * 1 + RIGHT * 1.5, TEAL),
            (r"\textsc{Tasks}", DOWN * 1 + LEFT * 1.5, RED),
            (r"\textsc{Serial}", DOWN * 0.2 + LEFT * 2.5, PINK),
        ]

        subsystem_objects = []
        connections_to_ros = []

        for name, position, color in subsystems:
            box = RoundedRectangle(
                width=1.6, height=0.6, corner_radius=0.1, color=color, fill_opacity=0.2
            ).move_to(position)
            text = Tex(name, font_size=20, color=WHITE).move_to(box.get_center())

            # Connection line to ROS
            line = Line(
                ros_core.get_center(), box.get_center(), color=color, stroke_width=2
            )

            subsystem_objects.append((box, text))
            connections_to_ros.append(line)

        # Animate subsystems appearing
        for (box, text), line in zip(subsystem_objects, connections_to_ros):
            self.play(Create(line), FadeIn(box), Write(text), run_time=0.4)

        self.wait(0.5)

        # ===== LOWER LAYER =====
        # STM32 core
        stm32_box = RoundedRectangle(
            width=2.2, height=0.9, corner_radius=0.1, color=GREEN, fill_opacity=0.3
        ).shift(DOWN * 2)
        stm32_text = Tex(
            r"\texttt{STM32F4/H7} \\ \texttt{Embassy}", font_size=22, color=WHITE
        ).move_to(stm32_box.get_center())

        self.play(FadeIn(stm32_box), Write(stm32_text))
        self.wait(0.3)

        # Lower layer components
        lower_components = [
            (r"\texttt{IMU}", DOWN * 2.8 + LEFT * 2.5),
            (r"\texttt{GPS}", DOWN * 2.8 + LEFT * 1),
            (r"\texttt{Encoders}", DOWN * 2.8 + RIGHT * 1),
            (r"\texttt{MCPWM}", DOWN * 2.8 + RIGHT * 2.5),
        ]

        lower_objects = []
        connections_to_stm32 = []

        for name, position in lower_components:
            circle = Circle(radius=0.4, color=GREEN, fill_opacity=0.2).move_to(position)
            text = Tex(name, font_size=16, color=WHITE).move_to(circle.get_center())

            line = Line(
                stm32_box.get_center(), circle.get_center(), color=GREEN, stroke_width=2
            )

            lower_objects.append((circle, text))
            connections_to_stm32.append(line)

        # Animate lower components
        for (circle, text), line in zip(lower_objects, connections_to_stm32):
            self.play(Create(line), FadeIn(circle), Write(text), run_time=0.3)

        self.wait(0.5)

        # ===== SERIAL CONNECTION =====
        # Find Serial subsystem and connect to STM32
        serial_box = subsystem_objects[5][0]  # Serial is 6th subsystem
        serial_connection = DashedLine(
            serial_box.get_bottom(),
            stm32_box.get_top(),
            color=WHITE,
            stroke_width=3,
            dash_length=0.1,
        )
        serial_label = (
            Tex(r"\texttt{UART} \\ $50~\mathrm{Hz}$", font_size=14, color=WHITE)
            .next_to(serial_connection, LEFT, buff=0.1)
            .shift(DOWN * 0.3)
        )

        self.play(Create(serial_connection), Write(serial_label))
        self.wait(1)

        # ===== DATA FLOW ANIMATION =====
        flow_title = Tex(
            r"\textbf{Guide Mode:} \textit{Data Flow}", font_size=32, color=YELLOW
        ).to_edge(UP)
        self.play(Transform(title, flow_title))
        self.wait(0.5)

        # Create data flow path
        flow_sequence = [
            (1, r"\textsc{Audio:} Wake word detected", ORANGE),
            (4, r"\textsc{Tasks:} Dispatch guide mission", RED),
            (3, r"\textsc{Navign:} Fetch route from server", TEAL),
            (2, r"\textsc{Bluetooth:} BLE localization", PURPLE),
            (0, r"\textsc{Vision:} AprilTag $+$ YOLO", YELLOW),
            (4, r"\textsc{Tasks:} LLM reasoning", RED),
            (1, r"\textsc{Audio:} Announce to user", ORANGE),
            (5, r"\textsc{Serial:} Send velocity cmd", PINK),
        ]

        # Animated data packets
        for idx, label, color in flow_sequence:
            box = subsystem_objects[idx][0]

            # Flash the subsystem
            self.play(box.animate.set_fill(color, opacity=0.6), run_time=0.2)

            # Show description
            desc = Tex(label, font_size=20, color=color).next_to(box, DOWN, buff=0.2)
            self.play(Write(desc), run_time=0.4)

            # Reset and remove description
            self.play(
                box.animate.set_fill(color, opacity=0.2), FadeOut(desc), run_time=0.3
            )

        # Final signal to STM32
        self.play(stm32_box.animate.set_fill(GREEN, opacity=0.6), run_time=0.3)

        stm32_desc = Tex(
            r"\textit{Execute motor control}", font_size=20, color=GREEN
        ).next_to(stm32_box, DOWN, buff=0.2)
        self.play(Write(stm32_desc))
        self.wait(0.5)

        # Animate motor output
        mcpwm_circle = lower_objects[3][0]  # MCPWM
        self.play(mcpwm_circle.animate.set_fill(GREEN, opacity=0.6), run_time=0.3)

        motor_arrows = VGroup(
            Arrow(
                mcpwm_circle.get_bottom(),
                mcpwm_circle.get_bottom() + DOWN * 0.5,
                color=GREEN,
            ),
            Arrow(
                mcpwm_circle.get_right(),
                mcpwm_circle.get_right() + RIGHT * 0.5,
                color=GREEN,
            ),
        )
        motor_label = Tex(r"\textit{Wheels turn}", font_size=16, color=GREEN).next_to(
            motor_arrows, DOWN
        )

        self.play(Create(motor_arrows), Write(motor_label))
        self.wait(1)

        # Reset
        self.play(
            stm32_box.animate.set_fill(GREEN, opacity=0.3),
            mcpwm_circle.animate.set_fill(GREEN, opacity=0.2),
            FadeOut(stm32_desc),
            FadeOut(motor_arrows),
            FadeOut(motor_label),
        )

        self.wait(0.5)

        # ===== HIGHLIGHT DUAL PURPOSE =====
        dual_title = Tex(
            r"\textbf{Dual Purpose:} \textit{Same Architecture}",
            font_size=32,
            color=GOLD,
        ).to_edge(UP)
        self.play(Transform(title, dual_title))

        # Mode indicators
        guide_mode = Tex(r"\textsc{Guide Mode}", font_size=24, color=BLUE).move_to(
            LEFT * 4 + UP * 3.5
        )
        delivery_mode = Tex(r"\textsc{Delivery Mode}", font_size=24, color=RED).move_to(
            RIGHT * 4 + UP * 3.5
        )

        self.play(Write(guide_mode), Write(delivery_mode))
        self.wait(0.3)

        # Show priority differences
        guide_priority = (
            VGroup(
                Tex(r"\textbf{Priority:}", font_size=16, color=BLUE),
                Tex(r"$1.$ Audio (user)", font_size=14),
                Tex(r"$2.$ Vision (safety)", font_size=14),
                Tex(r"$3.$ Navign (route)", font_size=14),
            )
            .arrange(DOWN, aligned_edge=LEFT, buff=0.15)
            .next_to(guide_mode, DOWN, buff=0.3)
        )

        delivery_priority = (
            VGroup(
                Tex(r"\textbf{Priority:}", font_size=16, color=RED),
                Tex(r"$1.$ Navign (efficiency)", font_size=14),
                Tex(r"$2.$ Bluetooth (coverage)", font_size=14),
                Tex(r"$3.$ Vision (obstacles)", font_size=14),
            )
            .arrange(DOWN, aligned_edge=LEFT, buff=0.15)
            .next_to(delivery_mode, DOWN, buff=0.3)
        )

        self.play(Write(guide_priority), Write(delivery_priority))
        self.wait(2)

        # Final message
        final_msg = Tex(
            r"\textit{One Platform, Two Missions, Six Subsystems}",
            font_size=28,
            color=GOLD,
        ).to_edge(DOWN)

        self.play(Write(final_msg))
        self.wait(2)

        # Fade out
        self.play(*[FadeOut(mob) for mob in self.mobjects])
        self.wait(0.5)


class DataFlowDetailed(Scene):
    def construct(self):
        # Title
        title = Tex(
            r"\textbf{Guide Scenario:} \texttt{``Take me to Starbucks''}", font_size=36
        ).to_edge(UP)
        self.play(Write(title))
        self.wait(0.5)

        # Create vertical flow chart
        steps = [
            (
                r"$\mathbf{1.}$ \textsc{Audio Wake Word}",
                ORANGE,
                r"\texttt{User: `Take me to Starbucks'}",
            ),
            (
                r"$\mathbf{2.}$ \textsc{Tasks Dispatcher}",
                RED,
                r"Dispatch guide mission",
            ),
            (
                r"$\mathbf{3.}$ \textsc{Navign Server}",
                TEAL,
                r"Route: $\text{Current} \to \text{Starbucks}$",
            ),
            (
                r"$\mathbf{4.}$ \textsc{Bluetooth Scan}",
                PURPLE,
                r"BLE: Zone A3, near escalator",
            ),
            (
                r"$\mathbf{5.}$ \textsc{Vision AprilTag}",
                YELLOW,
                r"Precise pose at intersection",
            ),
            (
                r"$\mathbf{6.}$ \textsc{Vision YOLO}",
                YELLOW,
                r"Detect: Person at $d = 2~\mathrm{m}$",
            ),
            (r"$\mathbf{7.}$ \textsc{Tasks LLM}", RED, r"Reason: Slow down, be polite"),
            (
                r"$\mathbf{8.}$ \textsc{Audio Output}",
                ORANGE,
                r"\texttt{`Excuse me, passing left'}",
            ),
            (
                r"$\mathbf{9.}$ \textsc{Serial UART}",
                PINK,
                r"Cmd: $v = 0.3~\mathrm{m/s}$, $\omega = -0.1~\mathrm{rad/s}$",
            ),
            (r"$\mathbf{10.}$ \textsc{STM32 Execute}", GREEN, r"MCPWM motor control"),
        ]

        # Starting position
        y_start = 3
        y_step = 0.65

        boxes = []
        arrows = []
        descriptions = []

        for i, (step_name, color, description) in enumerate(steps):
            y_pos = y_start - i * y_step

            # Step box
            box = RoundedRectangle(
                width=3.5, height=0.5, corner_radius=0.05, color=color, fill_opacity=0.3
            ).move_to(LEFT * 3 + UP * y_pos)

            step_text = Tex(step_name, font_size=16, color=color).move_to(
                box.get_center()
            )

            # Description
            desc_text = Tex(description, font_size=14, color=WHITE).next_to(
                box, RIGHT, buff=0.3
            )

            boxes.append((box, step_text))
            descriptions.append(desc_text)

            # Arrow to next step
            if i < len(steps) - 1:
                arrow = Arrow(
                    box.get_bottom(),
                    box.get_bottom() + DOWN * (y_step - 0.5),
                    color=WHITE,
                    stroke_width=2,
                    max_tip_length_to_length_ratio=0.15,
                )
                arrows.append(arrow)

        # Animate flow
        for i, ((box, step_text), desc_text) in enumerate(zip(boxes, descriptions)):
            self.play(FadeIn(box), Write(step_text), Write(desc_text), run_time=0.4)

            if i < len(arrows):
                self.play(Create(arrows[i]), run_time=0.2)

        self.wait(2)

        # Highlight parallel processing
        parallel_note = Tex(
            r"\textit{Note:} Bluetooth and Vision run continuously in parallel",
            font_size=18,
            color=YELLOW,
        ).to_edge(DOWN)

        self.play(Write(parallel_note))

        # Flash Bluetooth and Vision boxes
        bt_box = boxes[3][0]
        vision_boxes = [boxes[4][0], boxes[5][0]]

        for _ in range(2):
            self.play(
                bt_box.animate.set_fill(PURPLE, opacity=0.7),
                *[vb.animate.set_fill(YELLOW, opacity=0.7) for vb in vision_boxes],
                run_time=0.3,
            )
            self.play(
                bt_box.animate.set_fill(PURPLE, opacity=0.3),
                *[vb.animate.set_fill(YELLOW, opacity=0.3) for vb in vision_boxes],
                run_time=0.3,
            )

        self.wait(2)

        # Fade out
        self.play(*[FadeOut(mob) for mob in self.mobjects])
        self.wait(0.5)


class LocalizationHierarchy(Scene):
    def construct(self):
        # Title
        title = Tex(r"\textbf{Multi-Tier Localization Strategy}", font_size=36).to_edge(
            UP
        )
        self.play(Write(title))
        self.wait(0.5)

        # Three tiers
        tiers = [
            (
                r"\textsc{Tier 1: BLE Beacons}",
                r"\text{Coarse Localization}",
                r"$\pm 1{-}3~\mathrm{m}$",
                r"$f = 0.2{-}1~\mathrm{Hz}$",
                PURPLE,
                UP * 1.5,
            ),
            (
                r"\textsc{Tier 2: AprilTags}",
                r"\text{Precise Localization}",
                r"$\pm 2{-}5~\mathrm{cm}$",
                r"$f = 10{-}30~\mathrm{Hz}$",
                YELLOW,
                UP * 0,
            ),
            (
                r"\textsc{Tier 3: IMU + Odometry}",
                r"\text{Dead Reckoning}",
                r"$\text{Drift over time}$",
                r"$f = 100{-}200~\mathrm{Hz}$",
                GREEN,
                DOWN * 1.5,
            ),
        ]

        tier_objects = []

        for name, purpose, accuracy, rate, color, position in tiers:
            # Box
            box = RoundedRectangle(
                width=6, height=1.2, corner_radius=0.1, color=color, fill_opacity=0.2
            ).move_to(position)

            # Content
            content = (
                VGroup(
                    Tex(name, font_size=24, color=color),
                    Tex(purpose, font_size=18),
                    Tex(r"\textbf{Accuracy:}", accuracy, font_size=16),
                    Tex(r"\textbf{Rate:}", rate, font_size=16),
                )
                .arrange(DOWN, buff=0.1)
                .move_to(box.get_center())
            )

            tier_objects.append((box, content))

        # Animate tiers
        for box, content in tier_objects:
            self.play(FadeIn(box), Write(content), run_time=0.6)
            self.wait(0.3)

        self.wait(1)

        # Show fusion equation
        fusion_title = (
            Tex(r"\textbf{Extended Kalman Filter Fusion:}", font_size=24)
            .to_edge(DOWN)
            .shift(UP * 1.2)
        )

        fusion_eq = MathTex(
            r"\mathbf{x}_k = f(\mathbf{x}_{k-1}, \mathbf{u}_k) + \mathbf{w}_k",
            font_size=20,
        ).next_to(fusion_title, DOWN, buff=0.3)

        fusion_update = MathTex(
            r"\mathbf{x}_k \leftarrow \mathbf{x}_k + \mathbf{K}_k(\mathbf{z}_k - h(\mathbf{x}_k))",
            font_size=20,
        ).next_to(fusion_eq, DOWN, buff=0.2)

        state_vector = (
            MathTex(
                r"\mathbf{x} = \begin{bmatrix} x \\ y \\ \theta \\ v_x \\ v_y \\ \omega \end{bmatrix}",
                font_size=18,
            )
            .to_edge(DOWN)
            .shift(LEFT * 3)
        )

        state_label = Tex(r"\textit{State vector}", font_size=14).next_to(
            state_vector, DOWN, buff=0.1
        )

        self.play(Write(fusion_title))
        self.wait(0.3)
        self.play(Write(fusion_eq))
        self.wait(0.3)
        self.play(Write(fusion_update))
        self.wait(0.3)
        self.play(Write(state_vector), Write(state_label))

        self.wait(2)

        # Fade out
        self.play(*[FadeOut(mob) for mob in self.mobjects])
        self.wait(0.5)


class ControlLoop(Scene):
    def construct(self):
        # Title
        title = Tex(
            r"\textbf{Low-Level Control:} \texttt{STM32 + Embassy}", font_size=36
        ).to_edge(UP)
        self.play(Write(title))
        self.wait(0.5)

        # PID Control diagram
        pid_title = Tex(
            r"\textsc{PID Motor Control Loop} \quad ($f = 50~\mathrm{Hz}$)",
            font_size=24,
            color=BLUE,
        ).shift(UP * 2.5)

        self.play(Write(pid_title))
        self.wait(0.3)

        # Control blocks
        setpoint = RoundedRectangle(
            width=2, height=0.6, color=GREEN, fill_opacity=0.3
        ).shift(LEFT * 4 + UP * 1)
        setpoint_text = Tex(r"$v_{\text{target}}$", font_size=20).move_to(
            setpoint.get_center()
        )

        error_circle = Circle(radius=0.4, color=RED, fill_opacity=0.3).shift(
            LEFT * 2 + UP * 1
        )
        error_text = MathTex(
            r"e = v_{\text{target}} - v_{\text{actual}}", font_size=14
        ).move_to(error_circle.get_center())

        pid_box = RoundedRectangle(
            width=2.5, height=0.8, color=BLUE, fill_opacity=0.3
        ).shift(UP * 1)
        pid_text = MathTex(
            r"K_p e + K_i \int e \, dt + K_d \frac{de}{dt}", font_size=14
        ).move_to(pid_box.get_center())

        motor_box = RoundedRectangle(
            width=1.8, height=0.6, color=ORANGE, fill_opacity=0.3
        ).shift(RIGHT * 3 + UP * 1)
        motor_text = Tex(r"\texttt{MCPWM}", font_size=18).move_to(
            motor_box.get_center()
        )

        encoder_box = RoundedRectangle(
            width=1.8, height=0.6, color=PURPLE, fill_opacity=0.3
        ).shift(RIGHT * 3 + DOWN * 0.5)
        encoder_text = Tex(r"$v_{\text{actual}}$", font_size=18).move_to(
            encoder_box.get_center()
        )

        # Arrows
        arrow1 = Arrow(
            setpoint.get_right(), error_circle.get_left(), color=WHITE, stroke_width=3
        )
        arrow2 = Arrow(
            error_circle.get_right(), pid_box.get_left(), color=WHITE, stroke_width=3
        )
        arrow3 = Arrow(
            pid_box.get_right(), motor_box.get_left(), color=WHITE, stroke_width=3
        )
        arrow4 = Arrow(
            motor_box.get_bottom(), encoder_box.get_top(), color=WHITE, stroke_width=3
        )
        arrow5 = Arrow(
            encoder_box.get_left(),
            encoder_box.get_left() + LEFT * 4,
            color=WHITE,
            stroke_width=3,
        )

        # Feedback arrow
        feedback_points = [
            encoder_box.get_left() + LEFT * 0.5,
            encoder_box.get_left() + LEFT * 0.5 + DOWN * 0.5,
            error_circle.get_bottom() + DOWN * 0.5,
            error_circle.get_bottom(),
        ]
        feedback_arrow = VMobject(color=YELLOW, stroke_width=3)
        feedback_arrow.set_points_as_corners(feedback_points)

        # Animate control loop
        self.play(FadeIn(setpoint), Write(setpoint_text), run_time=0.4)
        self.play(Create(arrow1), run_time=0.3)
        self.play(FadeIn(error_circle), Write(error_text), run_time=0.4)
        self.play(Create(arrow2), run_time=0.3)
        self.play(FadeIn(pid_box), Write(pid_text), run_time=0.4)
        self.play(Create(arrow3), run_time=0.3)
        self.play(FadeIn(motor_box), Write(motor_text), run_time=0.4)
        self.play(Create(arrow4), run_time=0.3)
        self.play(FadeIn(encoder_box), Write(encoder_text), run_time=0.4)
        self.play(Create(feedback_arrow), run_time=0.5)

        self.wait(1)

        # IMU sensor fusion
        imu_title = Tex(
            r"\textsc{IMU Sensor Fusion:} \textit{Complementary Filter}",
            font_size=22,
            color=GREEN,
        ).shift(DOWN * 1.5)

        complementary_eq = MathTex(
            r"\theta = \alpha \cdot (\theta + \omega \cdot \Delta t) + (1-\alpha) \cdot \theta_{\text{accel}}",
            font_size=18,
        ).next_to(imu_title, DOWN, buff=0.3)

        alpha_note = Tex(
            r"\textit{where} $\alpha \approx 0.98$ (trust gyro for short term)",
            font_size=14,
        ).next_to(complementary_eq, DOWN, buff=0.2)

        self.play(Write(imu_title))
        self.wait(0.3)
        self.play(Write(complementary_eq))
        self.wait(0.3)
        self.play(Write(alpha_note))

        self.wait(2)

        # Fade out
        self.play(*[FadeOut(mob) for mob in self.mobjects])
        self.wait(0.5)
