from manim import *

class DatabaseSchema(Scene):
    def construct(self):
        # Title
        title = Text("Database Schema", font_size=24).to_edge(UP).shift(DOWN*0.3)
        self.add(title)
        
        # Collections as simple boxes
        entity = self.create_box("Entity", BLUE, "lon/lat range")
        area = self.create_box("Area", GREEN, "polygon: WKT")
        connection = self.create_box("Connection", ORANGE, "type + areas")
        merchant = self.create_box("Merchant", PURPLE, "location: WKT")
        beacon = self.create_box("Beacon", RED, "MAC + coords")
        
        # Layout: Entity at center, others around it
        entity.move_to(ORIGIN)
        area.move_to(LEFT*3 + UP*1)
        connection.move_to(LEFT*3 + DOWN*1)
        merchant.move_to(RIGHT*3 + UP*1)
        beacon.move_to(RIGHT*3 + DOWN*1)
        
        collections = VGroup(entity, area, connection, merchant, beacon)
        
        # References (arrows)
        refs = VGroup(
            Arrow(entity.get_left(), area.get_right(), buff=0.1, stroke_width=2, color=GRAY),
            Arrow(entity.get_left(), connection.get_right(), buff=0.1, stroke_width=2, color=GRAY),
            Arrow(entity.get_right(), merchant.get_left(), buff=0.1, stroke_width=2, color=GRAY),
            Arrow(entity.get_right(), beacon.get_left(), buff=0.1, stroke_width=2, color=GRAY),
            Arrow(area.get_bottom(), merchant.get_top(), buff=0.1, stroke_width=1.5, color=GRAY_A),
            Arrow(area.get_bottom(), beacon.get_left(), buff=0.1, stroke_width=1.5, color=GRAY_A),
        )
        
        # Animate (5 seconds total)
        self.play(FadeIn(collections), run_time=1.5)  # 1.5s: show boxes
        self.play(LaggedStart(*[GrowArrow(a) for a in refs], lag_ratio=0.1), run_time=1.5)  # 1.5s: draw arrows
        
        # Highlight WKT
        wkt_note = Text("WKT: Well-Known Text\n(mobile storage format)", 
                       font_size=14, color=YELLOW).to_corner(DR).shift(UP*0.3 + LEFT*0.3)
        wkt_highlights = VGroup(
            area[0].copy().set_stroke(YELLOW, width=3),
            merchant[0].copy().set_stroke(YELLOW, width=3)
        )
        
        self.play(Create(wkt_highlights), FadeIn(wkt_note), run_time=1.5)  # 1.5s: WKT highlight
        self.wait(0.5)  # 0.5s: hold
        
    def create_box(self, name, color, detail):
        """Create a collection box"""
        box = Rectangle(width=2, height=0.9, color=color, stroke_width=2, fill_opacity=0.1)
        name_text = Text(name, font_size=16, color=color, weight=BOLD)
        detail_text = Text(detail, font_size=10, color=GRAY)
        
        content = VGroup(name_text, detail_text).arrange(DOWN, buff=0.1)
        content.move_to(box.get_center())
        
        return VGroup(box, content)