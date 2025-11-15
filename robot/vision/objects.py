import cv2
import torch.nn


def detect_objects(model: torch.nn.Module, frame: cv2.typing.MatLike):
    image = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)

    detections = model(image)

    results = []

    for result in detections:
        xyxy = (
            result.boxes.xyxy
        )  # top-left-x, top-left-y, bottom-right-x, bottom-right-y
        names = [
            result.names[cls.item()] for cls in result.boxes.cls.int()
        ]  # class name of each box
        confs = result.boxes.conf  # confidence score of each box
        print(f"xyxy: {xyxy}, names: {names}, confs: {confs}")
        for [x1, y1, x2, y2], name, conf in zip(xyxy, names, confs):
            u = ((x1 + x2) / 2).item()
            v = ((y1 + y2) / 2).item()
            results.append((u, v, name, conf))

    return results
