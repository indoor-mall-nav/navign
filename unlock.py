from manim import *

class FullUnlockPipeline(Scene):
    def construct(self):
        # 0. Setup and Initialization (1.5s)
        title = Tex("Navign Secure Unlock Pipeline (Rust Code Flow)").scale(0.65).to_edge(UP)
        
        # Define Actors (Placement based on logical flow)
        phone = Tex("Client (Tauri)").scale(0.6).to_edge(LEFT).shift(UP*1.5)
        beacon = Tex("Beacon (ESP32-C3)").scale(0.6).to_edge(RIGHT).shift(UP*1.5)
        server = Tex("Server (Rust Backend)").scale(0.6).move_to(ORIGIN).shift(UP*1.5)
        
        actors = VGroup(phone, beacon, server)
        self.play(Write(title), FadeIn(actors), run_time=1.0)
        self.wait(0.5)
        
        # 1. Device Discovery & Capability Check (2.5s)
        scan_text = Tex("1. Scan, Connect, and Subscribe").scale(0.4).next_to(phone, DOWN, buff=0.2).set_color(BLUE)
        self.play(Write(scan_text), phone.animate.set_color(BLUE), run_time=0.8)
        
        # 0x01 Packet Structure - FIXED POSITION
        packet_0x01 = self.create_packet_structure(
            "0x01: Device Request",
            [("ID", "0x01", 1)],
            BLUE
        ).scale(0.8).move_to(DOWN * 2.8)
        
        arrow_to_beacon = Arrow(phone.get_center(), beacon.get_center(), color=BLUE).shift(DOWN*0.3)
        self.play(FadeIn(packet_0x01), GrowArrow(arrow_to_beacon), run_time=0.6)
        self.wait(0.3)
        
        # 0x02 Packet Structure - REPLACE IN SAME POSITION
        packet_0x02 = self.create_packet_structure(
            "0x02: Device Response",
            [
                ("ID", "0x02", 1),
                ("Type", "0x01-0x04", 1),
                ("Caps", "Bitfield", 1),
                ("ObjectId", "24 bytes (string)", 24)
            ],
            YELLOW
        ).scale(0.8).move_to(DOWN * 2.8)
        
        arrow_to_phone = Arrow(beacon.get_center(), phone.get_center(), color=YELLOW).shift(DOWN*0.3)
        self.play(
            Transform(packet_0x01, packet_0x02),
            Transform(arrow_to_beacon, arrow_to_phone),
            run_time=0.8
        )
        self.wait(0.3)
        self.play(FadeOut(packet_0x01), FadeOut(arrow_to_beacon), FadeOut(scan_text), run_time=0.5)
        
        # 2. Beacon Nonce Challenge (BLE) (2.5s)
        nonce_text = Tex("2. Request Beacon Nonce").scale(0.4).next_to(phone, DOWN, buff=0.2).set_color(BLUE)
        self.play(Write(nonce_text), run_time=0.5)
        
        # 0x03 Packet Structure - FIXED POSITION
        packet_0x03 = self.create_packet_structure(
            "0x03: Nonce Request",
            [("ID", "0x03", 1)],
            BLUE
        ).scale(0.8).move_to(DOWN * 2.8)
        
        arrow_nonce_req = Arrow(phone.get_center(), beacon.get_center(), color=BLUE).shift(DOWN*0.3)
        self.play(FadeIn(packet_0x03), GrowArrow(arrow_nonce_req), run_time=0.6)
        self.wait(0.3)
        
        # 0x04 Packet Structure - REPLACE IN SAME POSITION
        packet_0x04 = self.create_packet_structure(
            "0x04: Nonce Response",
            [
                ("ID", "0x04", 1),
                ("Nonce", "16 bytes", 16),
                ("Verifier", "8 bytes", 8)
            ],
            YELLOW
        ).scale(0.8).move_to(DOWN * 2.8)
        
        arrow_nonce_resp = Arrow(beacon.get_center(), phone.get_center(), color=YELLOW).shift(DOWN*0.3)
        self.play(
            Transform(packet_0x03, packet_0x04),
            Transform(arrow_nonce_req, arrow_nonce_resp),
            run_time=0.8
        )
        self.wait(0.3)
        self.play(FadeOut(packet_0x03), FadeOut(arrow_nonce_req), FadeOut(nonce_text), run_time=0.5)
        
        # 3. Server Challenge Creation (HTTPS) (3.5s)
        https_text1 = Tex("3a. Create Server Challenge").scale(0.4).next_to(phone, DOWN, buff=0.2).set_color(GREEN)
        self.play(Write(https_text1), run_time=0.5)
        
        # POST request to create challenge with beacon nonce
        json_create_challenge = self.create_json_payload(
            "POST /api/entities/{entity}/beacons/{id}/unlocker",
            {
                "payload": "base64(beacon_nonce + beacon_verifier)"
            }
        ).scale(0.8).move_to(DOWN * 2.8)
        
        arrow_create_req = Arrow(phone.get_center(), server.get_center(), color=GREEN).shift(DOWN*0.5)
        self.play(GrowArrow(arrow_create_req), FadeIn(json_create_challenge), run_time=0.8)
        self.wait(0.5)
        
        # Server generates its own challenge nonce
        server_process_1 = Tex("Generate Server\\\\Challenge Nonce").scale(0.4).set_color(ORANGE).move_to(server.get_center()).shift(DOWN*0.3)
        self.play(
            Transform(server, server_process_1),
            json_create_challenge.animate.set_opacity(0.4),
            run_time=0.8
        )
        self.wait(0.4)
        
        # Server responds with challenge
        json_challenge_response = self.create_json_payload(
            "Response 200 OK",
            {
                "instance_id": "ch_7f3a9e2b1c4d5e6f",
                "challenge": "hex(server_nonce_16_bytes)"
            }
        ).scale(0.8).move_to(DOWN * 2.8)
        
        arrow_challenge_resp = Arrow(server.get_center(), phone.get_center(), color=GREEN).shift(DOWN*0.5)
        self.play(
            FadeOut(arrow_create_req),
            Transform(json_create_challenge, json_challenge_response),
            GrowArrow(arrow_challenge_resp),
            Transform(server, Tex("Server (Rust Backend)").scale(0.6).move_to(ORIGIN).shift(UP*1.5)),
            run_time=0.8
        )
        self.wait(0.5)
        self.play(FadeOut(arrow_challenge_resp), FadeOut(json_create_challenge), FadeOut(https_text1), run_time=0.3)
        
        # 4. Client Authentication (4.5s)
        https_text2 = Tex("3b. Client Signs Server Challenge").scale(0.4).next_to(phone, DOWN, buff=0.2).set_color(BLUE)
        self.play(Write(https_text2), run_time=0.5)
        
        # Client performs biometric auth and signs
        client_process = Tex("Biometric Auth\\\\+ ECDSA Sign\\\\Server Challenge").scale(0.35).set_color(BLUE).move_to(phone.get_center()).shift(DOWN*0.3)
        self.play(Transform(phone, client_process), run_time=0.8)
        self.wait(0.8)
        
        # Client sends signed challenge to server
        json_sign_request = self.create_json_payload(
            "PUT /api/entities/.../unlocker/{instance_id}/status",
            {
                "payload": "base64(client_signature)"
            }
        ).scale(0.8).move_to(DOWN * 2.8)
        
        arrow_sign_req = Arrow(phone.get_center(), server.get_center(), color=GREEN).shift(DOWN*0.5)
        self.play(
            GrowArrow(arrow_sign_req),
            FadeIn(json_sign_request),
            Transform(phone, Tex("Client (Tauri)").scale(0.6).to_edge(LEFT).shift(UP*1.5)),
            run_time=0.8
        )
        self.wait(0.5)
        
        # Server verifies and generates proof
        server_process_2 = Tex("Verify Client Signature\\\\+ Generate Proof\\\\+ Sign with Server Key").scale(0.35).set_color(RED).move_to(server.get_center()).shift(DOWN*0.3)
        self.play(
            Transform(server, server_process_2),
            json_sign_request.animate.set_opacity(0.4),
            run_time=1.0
        )
        self.wait(0.6)
        
        # Server sends proof back
        json_proof_response = self.create_json_payload(
            "Response 200 OK",
            {
                "proof": "base64(server_sig_64B + beacon_verifier_8B)"
            }
        ).scale(0.8).move_to(DOWN * 2.8)
        
        arrow_proof_resp = Arrow(server.get_center(), phone.get_center(), color=GREEN).shift(DOWN*0.5)
        self.play(
            FadeOut(arrow_sign_req),
            Transform(json_sign_request, json_proof_response),
            GrowArrow(arrow_proof_resp),
            Transform(server, Tex("Server (Rust Backend)").scale(0.6).move_to(ORIGIN).shift(UP*1.5)),
            run_time=0.8
        )
        self.wait(0.5)
        self.play(
            FadeOut(arrow_proof_resp), 
            FadeOut(json_sign_request),
            FadeOut(https_text2), 
            run_time=0.5
        )
        
        # 5. Final Proof Transmission (BLE) (2.5s)
        unlock_text = Tex("4. Transmit Unlock Proof to Beacon").scale(0.4).next_to(phone, DOWN, buff=0.2).set_color(BLUE)
        self.play(Write(unlock_text), run_time=0.5)
        
        # 0x05 Packet Structure - FIXED POSITION
        packet_0x05 = self.create_packet_structure(
            "0x05: Unlock Request (Full Proof)",
            [
                ("ID", "0x05", 1),
                ("SrvNonce", "16B", 16),
                ("ClientVal", "8B", 8),
                ("BeaconVer", "8B", 8),
                ("Timestamp", "8B", 8),
                ("ServerSig", "64B", 64)
            ],
            BLUE
        ).scale(0.7).move_to(DOWN * 2.8)
        
        arrow_unlock_req = Arrow(phone.get_center(), beacon.get_center(), color=BLUE).shift(DOWN*0.3)
        self.play(FadeIn(packet_0x05), GrowArrow(arrow_unlock_req), run_time=0.7)
        self.wait(0.5)
        
        # Beacon verification
        verify_text = Tex("Verify Server\\\\Signature").scale(0.35).set_color(ORANGE).move_to(beacon.get_center()).shift(DOWN*0.3)
        self.play(
            Transform(beacon, verify_text),
            packet_0x05.animate.set_opacity(0.4),
            run_time=0.6
        )
        self.wait(0.4)
        
        # 0x06 Packet Structure - REPLACE IN SAME POSITION
        packet_0x06 = self.create_packet_structure(
            "0x06: Unlock Response",
            [
                ("ID", "0x06", 1),
                ("Success", "0x01", 1),
                ("Error", "0x00", 1)
            ],
            GREEN
        ).scale(0.8).move_to(DOWN * 2.8)
        
        success_icon = Tex(r"\checkmark").set_color(GREEN).scale(2).move_to(beacon.get_center())
        arrow_unlock_resp = Arrow(beacon.get_center(), phone.get_center(), color=GREEN).shift(DOWN*0.3)
        self.play(
            Transform(packet_0x05, packet_0x06),
            Transform(arrow_unlock_req, arrow_unlock_resp),
            Transform(beacon, success_icon),
            run_time=0.8
        )
        self.wait(0.3)
        
        # 6. Report Outcome (1.5s)
        report_text = Tex("5. Report Outcome to Server").scale(0.4).next_to(phone, DOWN, buff=0.2).set_color(GREEN)
        self.play(Write(report_text), run_time=0.5)
        
        json_outcome = self.create_json_payload(
            "PUT /api/entities/.../unlocker/{instance_id}/outcome",
            {
                "success": True,
                "outcome": "InvalidSignature (0x00)"
            }
        ).scale(0.8).move_to(DOWN * 2.8)
        
        arrow_outcome = Arrow(phone.get_center(), server.get_center(), color=GREEN).shift(DOWN*0.5)
        self.play(
            Transform(packet_0x05, json_outcome),
            Transform(arrow_unlock_req, arrow_outcome),
            run_time=0.7
        )
        self.wait(0.5)
        
        # Final success message
        final_report = Tex("Unlock Successful - Access Granted").scale(0.5).next_to(json_outcome, DOWN, buff=0.3).set_color(GREEN)
        self.play(
            Write(final_report),
            FadeOut(report_text),
            run_time=0.8
        )
        
        self.wait(1.0)
        
        # Fade out everything
        self.play(
            FadeOut(packet_0x05),
            FadeOut(arrow_unlock_req),
            FadeOut(final_report),
            run_time=0.5
        )
    
    def create_packet_structure(self, title, fields, color):
        """
        Create a visual representation of a BLE packet structure.
        fields: list of tuples (field_name, value, byte_count)
        """
        packet_group = VGroup()
        
        # Title with background
        title_text = Text(title).scale(0.35).set_color(WHITE)
        title_bg = SurroundingRectangle(title_text, color=color, fill_opacity=0.3, buff=0.1, stroke_width=2)
        title_group = VGroup(title_bg, title_text)
        packet_group.add(title_group)
        
        # Create boxes for each field
        boxes = VGroup()
        labels = VGroup()
        
        current_x = 0
        for field_name, value, byte_count in fields:
            # Scale box width based on byte count (with a minimum width)
            box_width = max(0.5, byte_count * 0.09)
            box = Rectangle(
                width=box_width, 
                height=0.5, 
                color=color, 
                stroke_width=2,
                fill_opacity=0.1,
                fill_color=color
            )
            box.shift(RIGHT * current_x)
            
            # Field name
            name_text = Text(field_name).scale(0.25).move_to(box.get_center()).shift(UP*0.12)
            
            # Value/size
            value_text = Text(value).scale(0.18).move_to(box.get_center()).shift(DOWN*0.1)
            
            boxes.add(box)
            labels.add(name_text, value_text)
            
            current_x += box_width
        
        # Center the boxes
        boxes.move_to(ORIGIN)
        labels.move_to(boxes.get_center())
        
        packet_visual = VGroup(boxes, labels)
        packet_visual.next_to(title_group, DOWN, buff=0.2)
        
        packet_group.add(packet_visual)
        
        return packet_group
    
    def create_json_payload(self, endpoint, data):
        """
        Create a visual representation of JSON payload
        """
        json_group = VGroup()
        
        # Endpoint header with background
        endpoint_text = Text(endpoint).scale(0.4).set_color(WHITE)
        endpoint_bg = SurroundingRectangle(endpoint_text, color=GREEN, fill_opacity=0.3, buff=0.15, stroke_width=2)
        endpoint_group = VGroup(endpoint_bg, endpoint_text)
        json_group.add(endpoint_group)
        
        # JSON structure
        json_lines = ["{"]
        for i, (key, value) in enumerate(data.items()):
            comma = "," if i < len(data) - 1 else ""
            if isinstance(value, str):
                json_lines.append(f'  "{key}": "{value}"{comma}')
            elif isinstance(value, bool):
                json_lines.append(f'  "{key}": {str(value).lower()}{comma}')
            else:
                json_lines.append(f'  "{key}": {value}{comma}')
        json_lines.append("}")
        
        json_text = Text("\n".join(json_lines), font="Courier", line_spacing=0.8).scale(0.35)
        json_text.next_to(endpoint_group, DOWN, buff=0.15)
        
        # Background box for JSON
        json_box = SurroundingRectangle(
            json_text, 
            color=GREEN, 
            buff=0.2, 
            stroke_width=2,
            fill_opacity=0.1,
            fill_color=GREEN
        )
        
        json_group.add(json_box, json_text)
        
        return json_group