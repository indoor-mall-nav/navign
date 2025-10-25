from manim import (
    Tex,
    Scene,
    VGroup,
    Circle,
    Rectangle,
    Dot,
    Write,
    FadeIn,
    FadeOut,
    Create,
    DashedLine,
    VMobject,
    Polygon,
    Arrow,
    DR,
    UP,
    DOWN,
    LEFT,
    RIGHT,
    ORANGE,
    BLUE,
    GREEN,
    RED,
    YELLOW,
    GRAY,
    WHITE,
)
import numpy as np


class PathfindingVisualization(Scene):
    def construct(self):
        # Title
        title = Tex("Navign Two-Tier Pathfinding").scale(0.6).to_edge(UP)
        self.play(Write(title), run_time=0.5)

        # Part 1: High-level area routing (Server) - 3.5s
        self.area_level_routing()

        # Part 2: Polygon quantification and inner routing - 5.5s
        self.polygon_quantification_and_routing()

        self.wait(0.5)

    def area_level_routing(self):
        """Show high-level area routing on server"""
        server_label = (
            Tex("SERVER: Area-Level Routing", font_size=20, color=RED)
            .to_edge(UP)
            .shift(DOWN * 0.5)
        )
        self.play(Write(server_label), run_time=0.4)

        # Three areas
        area_a = Circle(radius=0.5, color=BLUE, fill_opacity=0.2).shift(LEFT * 3)
        area_b = Circle(radius=0.5, color=GREEN, fill_opacity=0.2)
        area_c = Circle(radius=0.5, color=ORANGE, fill_opacity=0.2).shift(RIGHT * 3)

        label_a = Tex("A", font_size=28, color=BLUE).move_to(area_a)
        label_b = Tex("B", font_size=28, color=GREEN).move_to(area_b)
        label_c = Tex("C", font_size=28, color=ORANGE).move_to(area_c)

        # Connections
        conn_ab = Arrow(
            area_a.get_right(), area_b.get_left(), color=WHITE, buff=0.1, stroke_width=2
        )
        conn_bc = Arrow(
            area_b.get_right(), area_c.get_left(), color=WHITE, buff=0.1, stroke_width=2
        )

        areas = VGroup(area_a, area_b, area_c, label_a, label_b, label_c)
        conns = VGroup(conn_ab, conn_bc)

        self.play(FadeIn(areas), Create(conns), run_time=0.6)

        # Highlight connectivity graph
        graph_highlight = VGroup(
            conn_ab.copy().set_stroke(YELLOW, width=4),
            conn_bc.copy().set_stroke(YELLOW, width=4),
        )
        route_text = Tex("Route: A → B → C", font_size=18, color=YELLOW).next_to(
            areas, DOWN, buff=0.5
        )

        self.play(Create(graph_highlight), Write(route_text), run_time=0.7)
        self.wait(0.4)

        # Fade out
        self.play(
            FadeOut(areas),
            FadeOut(conns),
            FadeOut(graph_highlight),
            FadeOut(route_text),
            FadeOut(server_label),
            run_time=0.4,
        )

    def polygon_quantification_and_routing(self):
        """Show polygon quantification process and inner routing"""
        process_label = (
            Tex("Inner-Area: Polygon → Blocks → A* Path", font_size=18, color=GREEN)
            .to_edge(UP)
            .shift(DOWN * 0.5)
        )
        server_note = Tex("(Server or Mobile)", font_size=12, color=GRAY).next_to(
            process_label, DOWN, buff=0.1
        )
        self.play(Write(process_label), Write(server_note), run_time=0.4)

        # Step 1: Show polygon (1s)
        polygon_points = [
            (-2, -1, 0),
            (-2, 1.5, 0),
            (-0.5, 1.5, 0),
            (-0.5, 0.5, 0),
            (1, 0.5, 0),
            (1, 2, 0),
            (2.5, 2, 0),
            (2.5, -1, 0),
            (-2, -1, 0),
        ]

        polygon = Polygon(*polygon_points, color=BLUE, stroke_width=3, fill_opacity=0.1)
        poly_label = Tex("Polygon", font_size=14, color=BLUE).next_to(
            polygon, LEFT, buff=0.3
        )

        self.play(Create(polygon), Write(poly_label), run_time=0.7)

        # Step 2: Extract coordinates and show grid (1.5s)
        step1 = (
            Tex("1. Extract sorted X,Y coords", font_size=14, color=YELLOW)
            .to_corner(DR)
            .shift(UP * 2)
        )
        self.play(Write(step1), run_time=0.3)

        # Show grid lines
        x_coords = [-2, -0.5, 1, 2.5]
        y_coords = [-1, 0.5, 1.5, 2]

        grid_lines = VGroup()
        for x in x_coords:
            line = DashedLine([x, -1.2, 0], [x, 2.2, 0], color=GRAY, stroke_width=1)
            grid_lines.add(line)
        for y in y_coords:
            line = DashedLine([-2.2, y, 0], [2.7, y, 0], color=GRAY, stroke_width=1)
            grid_lines.add(line)

        self.play(Create(grid_lines), run_time=0.7)

        # Step 3: Test blocks with ray-casting (2s)
        step2 = Tex(
            "2. Ray-cast: test block centers", font_size=14, color=YELLOW
        ).next_to(step1, DOWN, buff=0.15, aligned_edge=LEFT)
        self.play(Write(step2), run_time=0.3)

        # Create blocks
        blocks = []
        for i in range(len(x_coords) - 1):
            for j in range(len(y_coords) - 1):
                x1, x2 = x_coords[i], x_coords[i + 1]
                y1, y2 = y_coords[j], y_coords[j + 1]
                center_x, center_y = (x1 + x2) / 2, (y1 + y2) / 2

                # Check if inside (simplified ray-casting visualization)
                is_inside = polygon.point_from_proportion(0)  # Placeholder
                # Manually determine based on the polygon shape
                is_inside = self.is_inside_polygon(
                    center_x, center_y, polygon_points[:-1]
                )

                block_rect = Rectangle(
                    width=x2 - x1,
                    height=y2 - y1,
                    stroke_width=1.5,
                    stroke_color=GREEN if is_inside else RED,
                    fill_opacity=0.3,
                    fill_color=GREEN if is_inside else RED,
                ).move_to([center_x, center_y, 0])

                blocks.append((block_rect, is_inside, (i, j)))

        # Animate block testing
        for block, is_inside, _ in blocks:
            self.play(FadeIn(block), run_time=0.05)

        self.wait(0.3)

        # Step 4: Show A* pathfinding (1.5s)
        step3 = Tex("3. A* on bounded blocks", font_size=14, color=YELLOW).next_to(
            step2, DOWN, buff=0.15, aligned_edge=LEFT
        )
        self.play(Write(step3), run_time=0.3)

        # Show a simple path
        start_point = np.array([-1.25, 0.25, 0])
        end_point = np.array([1.75, 1.25, 0])

        start_dot = Dot(start_point, color=YELLOW, radius=0.1)
        end_dot = Dot(end_point, color=YELLOW, radius=0.1)

        # Simple path through bounded blocks
        path_points = [
            start_point,
            np.array([-1.25, -0.25, 0]),
            np.array([0.25, -0.25, 0]),
            np.array([1.75, -0.25, 0]),
            np.array([1.75, 1.25, 0]),
            end_point,
        ]

        path_line = VMobject(color=YELLOW, stroke_width=5)
        path_line.set_points_as_corners(path_points)

        self.play(FadeIn(start_dot), FadeIn(end_dot), run_time=0.3)
        self.play(Create(path_line), run_time=0.8)

        # Final result
        result = Tex("Inner route computed", font_size=16, color=GREEN).to_edge(DOWN)
        self.play(Write(result), run_time=0.4)

        self.wait(0.5)

    def is_inside_polygon(self, x, y, points):
        """Ray-casting algorithm to check if point is inside polygon"""
        inside = False
        n = len(points)
        j = n - 1
        for i in range(n):
            xi, yi, _ = points[i]
            xj, yj, _ = points[j]
            if ((yi > y) != (yj > y)) and (x < (xj - xi) * (y - yi) / (yj - yi) + xi):
                inside = not inside
            j = i
        return inside
