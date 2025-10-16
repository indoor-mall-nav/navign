export function extractInstructions(data: ({
  'move': [number, number]
} | {
  'transport': [string, string, string]
})[]): Array<{
  type: 'straight' | 'turn' | 'transport' | 'unlock'
  straight?: number
  transport?: [string, string, string]
  turn?: 'left' | 'right' | 'around'
}> {
  const instructions: Array<{
    type: 'straight' | 'turn' | 'transport' | 'unlock'
    straight?: number
    transport?: [string, string, string]
    turn?: 'left' | 'right' | 'around'
  }> = []

  let currentDirection: [number, number] | null = null

  let straightDistance = 0

  data.forEach((item, _idx) => {
    if ('move' in item) {
      const [x, y] = item.move
      if (currentDirection) {
        const [dx, dy] = currentDirection
        const newDirection: [number, number] = [x - dx, y - dy]

        if (newDirection[0] === 0 && newDirection[1] === 0) {
          // No movement
          return
        }

        const crossProduct = dx * newDirection[1] - dy * newDirection[0]
        const dotProduct = dx * newDirection[0] + dy * newDirection[1]

        if (crossProduct === 0) {
          // Moving straight
          straightDistance += Math.sqrt(newDirection[0] ** 2 + newDirection[1] ** 2)
        } else {
          // Direction change
          if (straightDistance > 0) {
            instructions.push({ type: 'straight', straight: straightDistance })
            straightDistance = 0
          }

          if (dotProduct === 0) {
            instructions.push({ type: 'turn', turn: 'around' })
          } else if (crossProduct > 0) {
            instructions.push({ type: 'turn', turn: 'left' })
          } else {
            instructions.push({ type: 'turn', turn: 'right' })
          }

          straightDistance = Math.sqrt(newDirection[0] ** 2 + newDirection[1] ** 2)
        }

        currentDirection = newDirection
      } else {
        currentDirection = [x, y]
        straightDistance = 0
      }
    } else if ('transport' in item) {
      if (straightDistance > 0) {
        instructions.push({ type: 'straight', straight: straightDistance })
        straightDistance = 0
      }
      instructions.push({ type: 'transport', transport: item.transport } )
      currentDirection = null
    }
  })

  if (straightDistance > 0) {
    instructions.push({ type: 'straight', straight: straightDistance })
  }

  // Add unlock step at the end of navigation
  instructions.push({ type: 'unlock' })

  return instructions
}
