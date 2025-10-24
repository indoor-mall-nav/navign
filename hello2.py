from manim import *

class FullUnlockPipeline(Scene):
    def create_packet_block(self, id_hex, field_data, total_size, color=ORANGE):
        """
        Generates a structured block visualization of a packet.
        field_data: List of (name, size, field_color) tuples.
        """
        # Scale factor: 1 unit in Manim = approx 10 bytes for visualization clarity
        # Max width of the entire block visualization is capped at 5 units
        total_byte_length = total_size
        scale_factor = 5.0 / max(total_byte_length, 25) 
        
        # Base Label (Total Size)
        # This label is not used in the final animation movement, but kept for context if needed
        total_label = Text(f"ID: {id_hex} ({total_size} B)").scale(0.35).set_color(color).to_edge(UP).shift(LEFT*2.5)

        blocks = VGroup()
        labels = VGroup()
        # Start centered on the screen before being moved
        current_x = -total_byte_length * scale_factor / 2.0 
        
        for name, size, field_color in field_data:
            width = size * scale_factor
            
            # Create the block rectangle
            # FIX: Removed corner_radius=0.1 as it causes TypeError in this Manim version
            rect = Rectangle(width=width, height=0.5, color=field_color, fill_opacity=0.8).move_to([current_x + width/2, 0, 0])
            
            # Create the label for the field
            label = Text(f"{name} ({size}B)", font_size=18).set_color(field_color).next_to(rect, UP, buff=0.1)
            
            blocks.add(rect)
            labels.add(label)
            current_x += width

        packet_group = VGroup(blocks, labels).scale(0.8)
        
        # Add a title/summary label to the whole group
        title_summary = Text(f"Packet {id_hex}: {total_size} Bytes").scale(0.4).next_to(packet_group, DOWN, buff=0.5).set_color(color)
        
        return VGroup(packet_group, title_summary)

    def create_json_payload(self, fields, json_title_text, title_color=GREEN):
        """
        Generates a visualization of a JSON payload.
        fields: List of (key, value_description, value_color) tuples.
        """
        json_vgroup = VGroup()
        
        # JSON Start Brace
        start_brace = Text("{", font_size=20).set_color(title_color)
        json_vgroup.add(start_brace)

        # Key-Value Pairs
        key_value_pairs = VGroup()
        for i, (key, value_desc, f_color) in enumerate(fields):
            key_text = Text(f'"{key}": ', font_size=24).set_color(YELLOW_A)
            value_text = Text(f'"{value_desc}"', font_size=24).set_color(f_color)
            
            # Add comma if not the last item
            comma = Text(",", font_size=24).set_color(title_color) if i < len(fields) - 1 else Text("", font_size=24)
            
            line = VGroup(key_text, value_text, comma).arrange(RIGHT, aligned_edge=UP, buff=0.05)
            key_value_pairs.add(line)
        
        key_value_pairs.arrange(DOWN, aligned_edge=LEFT, buff=0.2).next_to(start_brace, DOWN, aligned_edge=LEFT, buff=0.1).shift(RIGHT*0.5)
        json_vgroup.add(key_value_pairs)
        
        # JSON End Brace
        end_brace = Text("}", font_size=20).set_color(title_color).next_to(key_value_pairs, DOWN, aligned_edge=LEFT, buff=0.1)
        json_vgroup.add(end_brace)
        
        # Arrange final group
        json_payload_group = VGroup(start_brace, key_value_pairs, end_brace).arrange(DOWN, aligned_edge=LEFT, buff=0.0)
        
        # Title
        json_title = Text(json_title_text).scale(0.4).set_color(title_color).next_to(json_payload_group, UP, buff=0.3)
        
        return VGroup(json_title, json_payload_group)


    def construct(self):
        # 0. Setup and Initialization (1.5s)
        title = Text("Navign Secure Unlock Pipeline").scale(0.65).to_edge(UP)
        
        # Define Actors (Placement based on logical flow)
        phone = Text("Client (Tauri)").scale(0.6).to_edge(LEFT).shift(UP*1.5)
        beacon = Text("Beacon (ESP32-C3)").scale(0.6).to_edge(RIGHT).shift(UP*1.5)
        server = Text("Server (Rust Backend)").scale(0.6).move_to(ORIGIN).shift(UP*1.5)
        
        actors = VGroup(phone, beacon, server)
        self.play(Write(title), FadeIn(actors), run_time=1.0)
        self.wait(0.5)

        # --- FIELD COLOR MAPPING ---
        ID_COLOR = RED
        DATA_COLOR = YELLOW_B
        CRYPTO_COLOR = BLUE_B
        METADATA_COLOR = GREEN_A
        
        # 1. Device Discovery & Capability Check (2.0s)
        scan_text = Text("1. Scan, Connect, and Subscribe").scale(0.4).next_to(phone, DOWN, buff=0.2).set_color(BLUE)
        self.play(Write(scan_text), phone.animate.set_color(BLUE), run_time=0.8)
        self.remove(scan_text)
        
        # 0x01: Device Request (1 B) - Simple, no complex block needed
        msg_1 = Text("0x01: Device Request (1 B)").scale(0.35).next_to(phone, DOWN, buff=0.5).set_color(ID_COLOR)
        self.play(msg_1.animate.move_to(beacon.get_center()).shift(DOWN*0.5), run_time=0.5)
        
        # 0x02: Device Response (27 B) - Structured Block
        fields_02 = [
            ("ID", 1, ID_COLOR),
            ("Type", 1, DATA_COLOR),
            ("Caps", 1, DATA_COLOR),
            ("Object ID", 24, METADATA_COLOR),
        ]
        packet_02 = self.create_packet_block("0x02", fields_02, 27, color=YELLOW).move_to(beacon.get_center()).shift(DOWN*0.5)
        self.play(
            Transform(msg_1, packet_02),
            msg_1.animate.move_to(phone.get_center()).shift(DOWN*0.7), 
            run_time=1.0
        )
        self.play(FadeOut(msg_1), run_time=0.5)
        
        # 2. Nonce Challenge (BLE) (1.5s)
        # 0x03: Nonce Request (1 B) - Simple
        msg_3 = Text("2. 0x03: Nonce Request (1 B)").scale(0.35).next_to(phone, DOWN, buff=0.5).set_color(ID_COLOR)
        self.play(msg_3.animate.move_to(beacon.get_center()).shift(DOWN*0.5), run_time=0.5)
        
        # 0x04: Nonce Response (25 B) - Structured Block
        fields_04 = [
            ("ID", 1, ID_COLOR),
            ("Nonce", 16, CRYPTO_COLOR),
            ("Tail", 8, CRYPTO_COLOR),
        ]
        packet_04 = self.create_packet_block("0x04", fields_04, 25, color=YELLOW).move_to(beacon.get_center()).shift(DOWN*0.5)

        self.play(
            Transform(msg_3, packet_04), 
            msg_3.animate.move_to(phone.get_center()).shift(DOWN*0.7), 
            run_time=1.0
        )
        self.play(FadeOut(msg_3), run_time=0.5)
        
        # 3. Server Challenge & User Authentication (HTTPS) (3.5s)
        
        # JSON Payload Visualization (Client to Server)
        json_fields = [
            ("instance_id", "24 B (Obj ID)", METADATA_COLOR),
            ("nonce", "16 B (Challenge)", CRYPTO_COLOR),
        ]
        json_payload_group = self.create_json_payload(
            json_fields, 
            "3. HTTPS JSON: Nonce Challenge Request"
        ).scale(0.7).next_to(phone, DOWN, buff=0.5)
        
        self.play(FadeIn(json_payload_group), run_time=0.7)

        # Move JSON to Server
        self.play(
            json_payload_group.animate.move_to(server.get_center()).shift(DOWN*0.5),
            run_time=1.0
        )
        
        # Server processing (Biometric, Signing)
        server_process = Text("SERVER: Validate User + ECDSA Signing").scale(0.5).set_color(RED).move_to(server.get_center()).shift(DOWN*0.1)
        self.play(
            Transform(server, server_process), 
            FadeOut(json_payload_group), # Fade out the incoming request
            run_time=1.0
        )
        
        # Server sends Proof (HTTPS) - Simple label for response
        https_label_2 = Text("HTTPS: Server Proof (Signature + Verifier)").scale(0.35).next_to(server, DOWN, buff=0.5).set_color(GREEN)
        self.play(
            FadeIn(https_label_2),
            https_label_2.animate.move_to(phone.get_center()).shift(DOWN*0.5),
            Transform(server, Text("Server (Rust Backend)").scale(0.6).move_to(ORIGIN).shift(UP*1.5)),
            run_time=1.0
        )
        self.wait(0.8)
        self.remove(https_label_2)
        
        # 4. Final Proof (BLE) (2.0s)
        # 0x05: UNLOCK_REQUEST (105 B) - Structured Block (Largest)
        fields_05 = [
            ("ID", 1, ID_COLOR),
            ("Nonce", 16, CRYPTO_COLOR),
            ("Dev Bytes", 8, DATA_COLOR),
            ("Verify", 8, CRYPTO_COLOR),
            ("Timestamp", 8, METADATA_COLOR),
            ("Server Signature", 64, CRYPTO_COLOR),
        ]
        packet_05 = self.create_packet_block("0x05", fields_05, 105, color=BLUE).move_to(phone.get_center()).shift(DOWN*0.7)
        
        self.play(
            FadeIn(packet_05),
            packet_05.animate.move_to(beacon.get_center()).shift(DOWN*0.7),
            run_time=1.0
        )
        
        # Beacon verification and success
        success_icon = Tex(r"\checkmark").set_color(GREEN).scale(1.5).move_to(beacon.get_center()).shift(DOWN*0.5)
        self.play(
            Transform(beacon, success_icon),
            FadeOut(packet_05),
            run_time=0.8
        )

        # 0x06: UNLOCK_RESPONSE (3 B) - Structured Block (Smallest)
        fields_06 = [
            ("ID", 1, ID_COLOR),
            ("Success", 1, GREEN),
            ("Error", 1, RED),
        ]
        packet_06 = self.create_packet_block("0x06", fields_06, 3, color=GREEN).move_to(beacon.get_center()).shift(DOWN*0.7)
        
        self.play(
            FadeIn(packet_06),
            packet_06.animate.move_to(phone.get_center()).shift(DOWN*0.7),
            run_time=0.5
        )
        
        # 5. Result Reporting & Cleanup (1.0s)
        final_report = Text("5. Report Result (Success) and Disconnect").scale(0.4).to_edge(DOWN).set_color(GREEN)
        self.play(
            Write(final_report),
            FadeOut(packet_06),
            run_time=1.0
        )
        
        self.wait(0.5)
