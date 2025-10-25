from manim import (
    Tex,
    Scene,
    VGroup,
    Line,
    Write,
    FadeIn,
    Create,
    LaggedStart,
    UP,
    DOWN,
    LEFT,
    RIGHT,
    BLUE,
    GREEN,
    RED,
    GRAY,
    WHITE,
)


class NavignIntro(Scene):
    def construct(self):
        # Main title (1.5s)
        title = Tex("NAVIGN", font_size=80, color=BLUE)
        subtitle = Tex("Indoor Navigation Where GPS Fails", font_size=28, color=GRAY)
        title_group = VGroup(title, subtitle).arrange(DOWN, buff=0.4)

        self.play(Write(title), run_time=0.7)
        self.play(FadeIn(subtitle), run_time=0.5)
        self.wait(0.3)

        # Problem + Solution (2.5s)
        self.play(title_group.animate.scale(0.6).to_edge(UP), run_time=0.4)
        self.show_problem_solution()

        self.wait(0.5)

    def show_problem_solution(self):
        """Show problem and solution side by side"""
        # Problem side
        problem_title = Tex("Problem", font_size=24, color=RED)
        problem_items = VGroup(
            Tex("GPS fails indoors", font_size=16, color=WHITE),
            Tex("Complex multi-floor routing", font_size=16, color=WHITE),
            Tex("Insecure access control", font_size=16, color=WHITE),
        ).arrange(DOWN, aligned_edge=LEFT, buff=0.2)

        problem_group = VGroup(problem_title, problem_items).arrange(DOWN, buff=0.3)
        problem_group.shift(LEFT * 3.2)

        # Solution side
        solution_title = Tex("Solution", font_size=24, color=GREEN)
        solution_items = VGroup(
            Tex("BLE beacon localization", font_size=16, color=WHITE),
            Tex("Two-tier A* pathfinding", font_size=16, color=WHITE),
            Tex("ECDSA + biometric auth", font_size=16, color=WHITE),
        ).arrange(DOWN, aligned_edge=LEFT, buff=0.2)

        solution_group = VGroup(solution_title, solution_items).arrange(DOWN, buff=0.3)
        solution_group.shift(RIGHT * 3.2)

        # Divider
        divider = Line(UP * 2, DOWN * 2, color=GRAY, stroke_width=2)

        # Animate
        self.play(
            Write(problem_title), Write(solution_title), Create(divider), run_time=0.5
        )
        self.play(
            LaggedStart(*[FadeIn(item) for item in problem_items], lag_ratio=0.2),
            LaggedStart(*[FadeIn(item) for item in solution_items], lag_ratio=0.2),
            run_time=1.5,
        )

        self.wait(0.5)
