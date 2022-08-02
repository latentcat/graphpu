struct Node {
  position: vec3<f32>,
  velocity: vec3<f32>,
};

struct SimParams {
  deltaT : f32,
  rule1Distance : f32,
  rule2Distance : f32,
  rule3Distance : f32,
  rule1Scale : f32,
  rule2Scale : f32,
  rule3Scale : f32,
};

struct Uniforms {
    frame_num: u32,
};

@group(0) @binding(0) var<uniform> params : SimParams;
@group(0) @binding(1) var<storage, read_write> nodeSrc : array<Node>;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;


fn hash(s: u32) -> u32 {
    var t : u32 = s;
    t ^= 2747636419u;
    t *= 2654435769u;
    t ^= t >> 16u;
    t *= 2654435769u;
    t ^= t >> 16u;
    t *= 2654435769u;
    return t;
}

fn random(seed: u32) -> f32 {
    return f32(hash(seed)) / 4294967295.0; // 2^32-1
}

fn random_xy(seed_x: u32, seed_y: u32) -> f32 {
  return f32(hash(hash(seed_x) + seed_y)) / 4294967295.0; // 2^32-1
}

// https://github.com/austinEng/Project6-Vulkan-Flocking/blob/master/data/shaders/computeparticles/particle.comp
@compute
@workgroup_size(128)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let total = arrayLength(&nodeSrc);
  let index = global_invocation_id.x;
  if (index >= total) {
    return;
  }

  var vPos : vec3<f32> = nodeSrc[index].position;
  var vVel : vec3<f32> = nodeSrc[index].velocity;

  var cMass : vec3<f32> = vec3<f32>(0.0);
  var cVel : vec3<f32> = vec3<f32>(0.0);
  var colVel : vec3<f32> = vec3<f32>(0.0);
  var cMassCount : i32 = 0;
  var cVelCount : i32 = 0;

  var i : u32 = 0u;
  loop {
    if (i >= total) {
      break;
    }
    if (i == index) {
      continue;
    }

    let pos = nodeSrc[i].position;
    let vel = nodeSrc[i].velocity;

    if (distance(pos, vPos) < params.rule1Distance) {
      cMass += pos;
      cMassCount += 1;
    }
    if (distance(pos, vPos) < params.rule2Distance) {
      colVel -= pos - vPos;
    }
    if (distance(pos, vPos) < params.rule3Distance) {
      cVel += vel;
      cVelCount += 1;
    }

    continuing {
      i = i + 1u;
    }
  }
  if (cMassCount > 0) {
    cMass = cMass * (1.0 / f32(cMassCount)) - vPos;
  }
  if (cVelCount > 0) {
    cVel *= 1.0 / f32(cVelCount);
  }

  vVel = vVel + (cMass * params.rule1Scale) +
      (colVel * params.rule2Scale) +
      (cVel * params.rule3Scale);

  // clamp velocity for a more pleasing simulation
  if (dot(vVel, vVel) > 0.0) {
    vVel = normalize(vVel) * clamp(length(vVel), 0.0, 0.1);
  }

  // kinematic update
  vPos += vVel * params.deltaT;

  // Wrap around boundary
  if (vPos.x < -1.0) {
    vPos.x = 1.0;
  }
  if (vPos.x > 1.0) {
    vPos.x = -1.0;
  }
  if (vPos.y < -1.0) {
    vPos.y = 1.0;
  }
  if (vPos.y > 1.0) {
    vPos.y = -1.0;
  }

  // Write back
  nodeSrc[index] = Node(vPos, vVel);
}


@compute
@workgroup_size(128)
fn randomize(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let total = arrayLength(&nodeSrc);
  let index = global_invocation_id.x;
  if (index >= total) {
    return;
  }

  var vPos : vec3<f32> = nodeSrc[index].position;
  var vVel : vec3<f32> = nodeSrc[index].velocity;

  vPos.x = random_xy(index, 0u + 3u * uniforms.frame_num) * 2.0 - 1.0;
  vPos.y = random_xy(index, 1u + 3u * uniforms.frame_num) * 2.0 - 1.0;
  vPos.z = 0.0;

  vVel = vec3<f32>(0.0);

  // Write back
  nodeSrc[index] = Node(vPos, vVel);
}

@compute
@workgroup_size(128)
fn copy(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let total = arrayLength(&nodeSrc);
  let index = global_invocation_id.x;
  if (index >= total) {
    return;
  }

  var vPos : vec3<f32> = nodeSrc[index].position;
  var vVel : vec3<f32> = nodeSrc[index].velocity;

  // Write back
//  nodeSrc[index] = Node(vPos, vVel);
}