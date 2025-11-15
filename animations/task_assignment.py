from manim import (
    Tex,
    Scene,
    VGroup,
    Circle,
    Rectangle,
    RoundedRectangle,
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
    TEAL,
    GRAY,
    WHITE,
)


class RobotTaskAssignment(Scene):
    def construct(self):
        # Title
        title = Tex("Robot Task Assignment Pipeline", font_size=32, color=TEAL).to_edge(
            UP
        )
        self.play(Write(title), run_time=0.5)

        # Part 1: System Architecture (2.5s)
        self.show_architecture()

        # Part 2: Task Submission (3s)
        self.show_task_submission()

        # Part 3: Robot Selection (4s)
        self.show_robot_selection()

        # Part 4: Task Execution (3.5s)
        self.show_task_execution()

        self.wait(0.5)

    def show_architecture(self):
        """Show distributed system components"""
        section_label = (
            Tex("Distributed Architecture", font_size=20, color=BLUE)
            .to_edge(UP)
            .shift(DOWN * 0.5)
        )
        self.play(Write(section_label), run_time=0.3)

        # Mobile app
        mobile = RoundedRectangle(
            width=1.5, height=1, corner_radius=0.1, color=BLUE, fill_opacity=0.2
        ).shift(LEFT * 5 + UP * 1)
        mobile_text = Tex("Mobile App", font_size=14, color=BLUE).move_to(
            mobile.get_center()
        )

        # Orchestrator
        orchestrator = RoundedRectangle(
            width=2, height=1.2, corner_radius=0.1, color=GREEN, fill_opacity=0.2
        ).shift(LEFT * 1.5 + UP * 1)
        orchestrator_text = (
            VGroup(
                Tex("Orchestrator", font_size=14, color=GREEN),
                Tex("(Rust gRPC)", font_size=10, color=GRAY),
            )
            .arrange(DOWN, buff=0.05)
            .move_to(orchestrator.get_center())
        )

        # Tower
        tower = RoundedRectangle(
            width=2, height=1.2, corner_radius=0.1, color=PURPLE, fill_opacity=0.2
        ).shift(RIGHT * 2 + UP * 1)
        tower_text = (
            VGroup(
                Tex("Tower", font_size=14, color=PURPLE),
                Tex("(Go Socket.IO)", font_size=10, color=GRAY),
            )
            .arrange(DOWN, buff=0.05)
            .move_to(tower.get_center())
        )

        # Robots
        robots = []
        robot_positions = [
            RIGHT * 4.5 + DOWN * 0.5,
            RIGHT * 5.5 + DOWN * 0.5,
            RIGHT * 4.5 + DOWN * 1.5,
            RIGHT * 5.5 + DOWN * 1.5,
        ]

        for i, pos in enumerate(robot_positions):
            robot_circle = Circle(radius=0.3, color=ORANGE, fill_opacity=0.3).move_to(
                pos
            )
            robot_label = Tex(f"R{i + 1}", font_size=10, color=WHITE).move_to(
                robot_circle.get_center()
            )
            robots.append((robot_circle, robot_label))

        # Connections
        mobile_to_orch = Arrow(
            mobile.get_right(),
            orchestrator.get_left(),
            color=WHITE,
            stroke_width=2,
            buff=0.1,
        )
        orch_to_tower = Arrow(
            orchestrator.get_right(),
            tower.get_left(),
            color=WHITE,
            stroke_width=2,
            buff=0.1,
        )

        tower_to_robots = []
        for robot_circle, _ in robots:
            arrow = Arrow(
                tower.get_right(),
                robot_circle.get_left(),
                color=WHITE,
                stroke_width=1.5,
                buff=0.1,
            )
            tower_to_robots.append(arrow)

        # Animate architecture
        self.play(
            FadeIn(mobile),
            Write(mobile_text),
            FadeIn(orchestrator),
            Write(orchestrator_text),
            FadeIn(tower),
            Write(tower_text),
            run_time=0.8,
        )

        self.play(Create(mobile_to_orch), Create(orch_to_tower), run_time=0.4)

        for (robot_circle, robot_label), arrow in zip(robots, tower_to_robots):
            self.play(
                FadeIn(robot_circle),
                Write(robot_label),
                Create(arrow),
                run_time=0.15,
            )

        self.wait(0.4)

        # Store components
        self.section_label = section_label
        self.mobile = mobile
        self.mobile_text = mobile_text
        self.orchestrator = orchestrator
        self.orchestrator_text = orchestrator_text
        self.tower = tower
        self.tower_text = tower_text
        self.robots = robots
        self.mobile_to_orch = mobile_to_orch
        self.orch_to_tower = orch_to_tower
        self.tower_to_robots = tower_to_robots

    def show_task_submission(self):
        """Show task submission from mobile to orchestrator"""
        self.play(
            Transform(
                self.section_label,
                Tex("Task Submission", font_size=20, color=BLUE)
                .to_edge(UP)
                .shift(DOWN * 0.5),
            ),
            run_time=0.3,
        )

        # Task details
        task_box = Rectangle(
            width=2.5, height=1.5, color=BLUE, stroke_width=2, fill_opacity=0.15
        ).shift(LEFT * 5 + DOWN * 1.5)

        task_content = (
            VGroup(
                Tex("Delivery Task", font_size=14, color=BLUE),
                Tex("From: Floor 1", font_size=10, color=WHITE),
                Tex("To: Floor 2", font_size=10, color=WHITE),
                Tex("Priority: High", font_size=10, color=YELLOW),
            )
            .arrange(DOWN, aligned_edge=LEFT, buff=0.08)
            .move_to(task_box.get_center())
        )

        self.play(FadeIn(task_box), Write(task_content), run_time=0.6)

        # Task flows to orchestrator
        task_arrow = Arrow(
            task_box.get_top(),
            self.orchestrator.get_bottom(),
            color=BLUE,
            stroke_width=3,
        )

        self.play(Create(task_arrow), run_time=0.5)

        # Orchestrator queues task
        queue_label = Tex(
            "Task queued (priority: High)", font_size=12, color=YELLOW
        ).next_to(self.orchestrator, DOWN, buff=0.2)

        self.play(Write(queue_label), run_time=0.5)

        self.wait(0.4)

        # Store for next scene
        self.task_box = task_box
        self.task_content = task_content
        self.task_arrow = task_arrow
        self.queue_label = queue_label

    def show_robot_selection(self):
        """Show robot selection algorithm"""
        self.play(
            Transform(
                self.section_label,
                Tex("Robot Selection Algorithm", font_size=20, color=GREEN)
                .to_edge(UP)
                .shift(DOWN * 0.5),
            ),
            FadeOut(self.task_box),
            FadeOut(self.task_content),
            FadeOut(self.task_arrow),
            FadeOut(self.queue_label),
            run_time=0.4,
        )

        # Show robot states
        robot_states = [
            ("R1: Idle, 85\\%", GREEN),
            ("R2: Busy, 60\\%", GRAY),
            ("R3: Idle, 45\\%", GREEN),
            ("R4: Charging, 25\\%", YELLOW),
        ]

        state_labels = []
        for i, ((robot_circle, robot_label), (state, color)) in enumerate(
            zip(self.robots, robot_states)
        ):
            state_text = Tex(state, font_size=9, color=color).next_to(
                robot_circle, DOWN, buff=0.1
            )
            state_labels.append(state_text)
            self.play(Write(state_text), run_time=0.2)

        self.wait(0.3)

        # Scoring criteria
        scoring_box = Rectangle(
            width=2.8, height=1.3, color=GREEN, stroke_width=2, fill_opacity=0.1
        ).shift(LEFT * 1.5 + DOWN * 1.5)

        scoring_content = (
            VGroup(
                Tex("Scoring Criteria:", font_size=12, color=GREEN),
                Tex("Battery: 0-100pts", font_size=9, color=WHITE),
                Tex("Proximity: 0-100pts", font_size=9, color=WHITE),
                Tex("Capability: 0-50pts", font_size=9, color=WHITE),
            )
            .arrange(DOWN, aligned_edge=LEFT, buff=0.08)
            .move_to(scoring_box.get_center())
        )

        self.play(FadeIn(scoring_box), Write(scoring_content), run_time=0.7)

        self.wait(0.4)

        # Calculate scores
        score_results = [
            ("R1: Score 165", GREEN),
            ("R2: Ineligible", RED),
            ("R3: Score 125", BLUE),
            ("R4: Ineligible", RED),
        ]

        score_labels = []
        for i, ((robot_circle, _), (score, color)) in enumerate(
            zip(self.robots, score_results)
        ):
            score_text = Tex(score, font_size=9, color=color).next_to(
                robot_circle, RIGHT, buff=0.15
            )
            score_labels.append(score_text)
            self.play(Write(score_text), run_time=0.2)

        self.wait(0.3)

        # Select best robot (R1)
        selected_highlight = self.robots[0][0].copy().set_stroke(YELLOW, width=4)
        selected_label = Tex("Selected!", font_size=12, color=YELLOW).next_to(
            self.robots[0][0], UP, buff=0.2
        )

        self.play(Create(selected_highlight), Write(selected_label), run_time=0.5)

        self.wait(0.5)

        # Store for next scene
        self.state_labels = state_labels
        self.scoring_box = scoring_box
        self.scoring_content = scoring_content
        self.score_labels = score_labels
        self.selected_highlight = selected_highlight
        self.selected_label = selected_label

    def show_task_execution(self):
        """Show task assignment and execution"""
        self.play(
            Transform(
                self.section_label,
                Tex("Task Assignment \\& Execution", font_size=20, color=ORANGE)
                .to_edge(UP)
                .shift(DOWN * 0.5),
            ),
            FadeOut(self.scoring_box),
            FadeOut(self.scoring_content),
            FadeOut(VGroup(*self.score_labels)),
            FadeOut(VGroup(*self.state_labels)),
            run_time=0.4,
        )

        # gRPC stream to tower
        grpc_arrow = Arrow(
            self.orchestrator.get_right(),
            self.tower.get_left(),
            color=GREEN,
            stroke_width=4,
        ).shift(UP * 0.1)

        grpc_label = Tex("gRPC stream", font_size=10, color=GREEN).next_to(
            grpc_arrow, UP, buff=0.05
        )

        self.play(Create(grpc_arrow), Write(grpc_label), run_time=0.5)

        # Socket.IO to robot
        socketio_arrow = self.tower_to_robots[0].copy().set_stroke(PURPLE, width=4)
        socketio_label = Tex("Socket.IO", font_size=10, color=PURPLE).next_to(
            socketio_arrow, UP, buff=0.05
        )

        self.play(Create(socketio_arrow), Write(socketio_label), run_time=0.5)

        self.wait(0.3)

        # Robot executes task
        execution_steps = [
            ("Pathfinding query", BLUE),
            ("Navigate to source", GREEN),
            ("Load item", ORANGE),
            ("Navigate to terminal", GREEN),
            ("Deliver item", YELLOW),
        ]

        execution_box = Rectangle(
            width=2.2, height=2, color=ORANGE, stroke_width=2, fill_opacity=0.1
        ).shift(RIGHT * 5 + DOWN * 2.8)

        execution_title = Tex("Execution Steps:", font_size=11, color=ORANGE).next_to(
            execution_box.get_top(), DOWN, buff=0.1
        )

        self.play(FadeIn(execution_box), Write(execution_title), run_time=0.4)

        y_offset = -0.2
        for step, color in execution_steps:
            step_text = Tex(f"\\checkmark {step}", font_size=9, color=color).move_to(
                execution_box.get_center() + DOWN * y_offset
            )
            self.play(Write(step_text), run_time=0.3)
            y_offset += 0.35

        self.wait(0.3)

        # Status updates back to orchestrator
        status_arrow = Arrow(
            self.robots[0][0].get_left(),
            self.orchestrator.get_right(),
            color=YELLOW,
            stroke_width=2,
            buff=0.1,
        ).shift(DOWN * 0.3)

        status_label = Tex("Status updates", font_size=9, color=YELLOW).next_to(
            status_arrow, DOWN, buff=0.05
        )

        self.play(Create(status_arrow), Write(status_label), run_time=0.5)

        # Task complete
        complete_text = Tex(
            "Task Complete: Delivery successful", font_size=16, color=GREEN
        ).to_edge(DOWN)

        self.play(Write(complete_text), run_time=0.5)

        self.wait(0.5)
