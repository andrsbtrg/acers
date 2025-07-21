from typing import List, Tuple

class Collision:
    ids: Tuple[str, str]
    point: Tuple[float, float, float]
    dist: float

def clash_detection(a: str, b: str, min_dist: float = 0) -> List[Collision]: ...
