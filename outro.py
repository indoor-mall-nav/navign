from manim import *

config.text_font = "Latin Modern Math"


class NavignOutro(Scene):
    def construct(self):
        # Title (0.5s)
        title = Text("Technical Architecture", font_size=32, color=BLUE).to_edge(UP)
        self.play(Write(title), run_time=0.5)
        
        # Architecture diagram (2.5s)
        self.show_architecture()
        
        # GitHub repos (2s)
        self.show_github()
        
        # Future plans (1.5s)
        self.show_future()
        
        self.wait(1)
    
    def show_architecture(self):
        """Show three-tier architecture with Rust emphasis"""
        # Rust badge
        rust_badge = VGroup(
            Tex("", font_size=28),
            Tex("100\% Rust", font_size=14, color=ORANGE)
        ).arrange(RIGHT, buff=0.15).to_edge(UP).shift(DOWN*0.8)
        
        self.play(FadeIn(rust_badge), run_time=0.3)
        
        # Three components
        mobile = self.create_component(
            "", "Mobile", 
            "Tauri framework\\\\Max Rust integration",
            BLUE
        ).shift(LEFT*3.5 + UP*0.2)
        
        server = self.create_component(
            "", "Server",
            "Axum + MongoDB\\\\Service trait pattern",
            GREEN
        ).shift(UP*0.2)
        
        beacon = self.create_component(
            "", "Beacon",
            "ESP32-C3\\\\esp-hal bare-metal",
            RED
        ).shift(RIGHT*3.5 + UP*0.2)
        
        components = VGroup(mobile, server, beacon)
        
        # Connections (using regular arrows, not CurvedArrow for GrowArrow)
        arrow1 = Arrow(mobile.get_right(), server.get_left(), buff=0.1, stroke_width=2, color=YELLOW)
        arrow2 = Arrow(server.get_right(), beacon.get_left(), buff=0.1, stroke_width=2, color=YELLOW)
        
        # Create curved arrow manually for BLE connection
        ble_path = CurvedArrow(
            mobile.get_top() + UP*0.1, 
            beacon.get_top() + UP*0.1, 
            angle=-TAU/6, 
            stroke_width=2, 
            color=PURPLE
        )
        
        labels = VGroup(
            Tex("HTTPS", font_size=10, color=YELLOW).next_to(arrow1, UP, buff=0.05),
            Tex("API", font_size=10, color=YELLOW).next_to(arrow2, UP, buff=0.05),
            Tex("BLE", font_size=10, color=PURPLE).next_to(ble_path, UP, buff=0.05)
        )
        
        self.play(LaggedStart(*[FadeIn(c) for c in components], lag_ratio=0.15), run_time=1.0)
        
        # Animate arrows separately (GrowArrow for straight arrows, Create for curved)
        self.play(
            GrowArrow(arrow1),
            GrowArrow(arrow2),
            Create(ble_path),
            run_time=0.6
        )
        self.play(
            LaggedStart(*[Write(l) for l in labels], lag_ratio=0.1),
            run_time=0.4
        )
        
        self.arch_group = VGroup(rust_badge, components, arrow1, arrow2, ble_path, labels)
    
    def show_github(self):
        """Show GitHub repositories"""
        github_box = Rectangle(width=11, height=1.6, color=WHITE, 
                              stroke_width=2, fill_opacity=0.05)
        github_box.next_to(self.arch_group, DOWN, buff=0.4)
        
        github_icon = Tex("Open Source", font_size=18, color=YELLOW)
        github_icon.next_to(github_box.get_top(), DOWN, buff=0.15)
        
        repos = VGroup(
            Tex("github.com/indoor-mall-nav/mobile", font_size=12, color=BLUE),
            Tex("github.com/indoor-mall-nav/server", font_size=12, color=GREEN),
            Tex("github.com/indoor-mall-nav/beacon", font_size=12, color=RED),
        ).arrange(DOWN, aligned_edge=LEFT, buff=0.12)
        repos.next_to(github_icon, DOWN, buff=0.15)
        
        self.play(
            Create(github_box),
            Write(github_icon),
            run_time=0.5
        )
        self.play(
            LaggedStart(*[Write(r) for r in repos], lag_ratio=0.2),
            run_time=1.2
        )
        
        self.github_group = VGroup(github_box, github_icon, repos)
    
    def show_future(self):
        """Show future roadmap"""
        future_box = Rectangle(width=11, height=1.3, color=GRAY, 
                              stroke_width=1.5, stroke_opacity=0.6, fill_opacity=0.03)
        future_box.next_to(self.github_group, DOWN, buff=0.3)
        
        future_title = Tex("Roadmap", font_size=16, 
                           color=GRAY)
        future_title.next_to(future_box.get_top(), DOWN, buff=0.12)
        
        plans = VGroup(
            Tex("Vision Pro AR navigation interface", font_size=11, color=GRAY),
            Tex("Autonomous delivery robot: rclrs + OpenCV + AprilTag + STM32 PID", 
                font_size=11, color=GRAY),
        ).arrange(DOWN, aligned_edge=LEFT, buff=0.1)
        plans.next_to(future_title, DOWN, buff=0.12)
        
        self.play(
            Create(future_box),
            Write(future_title),
            run_time=0.4
        )
        self.play(
            LaggedStart(*[Write(p) for p in plans], lag_ratio=0.3),
            run_time=1.0
        )
    
    def show_cta(self):
        """Final call to action"""
        cta = Tex("Ethan Wu – github.com/7086cmd", 
                  font_size=16, color=BLUE, slant=ITALIC)
        cta.to_edge(DOWN).shift(UP*0.3)
        
        self.play(FadeIn(cta), run_time=0.5)
    
    def create_component(self, icon, name, details, color):
        """Create a component box"""
        box = Rectangle(width=2.3, height=1.6, color=color, 
                       stroke_width=2, fill_opacity=0.1)
        
        icon_text = Tex(icon, font_size=32)
        name_text = Tex(name, font_size=15, color=color)
        detail_text = Tex(details, font_size=9, color=GRAY)
        
        content = VGroup(icon_text, name_text, detail_text).arrange(DOWN, buff=0.12)
        content.move_to(box.get_center())
        
        return VGroup(box, content)