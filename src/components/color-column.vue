<template>
  <div class="color-column">
    <slot name="top-button">
      <div
        v-if="withButtons"
        @click="$emit('min')"
        class="top-button"
      >
        Min
      </div>
    </slot>

    <canvas width="0" height="0" ref="picker" />
    
    <slot name="bottom-button">
      <div
        v-if="withButtons"
        @click="$emit('max')"
        class="bottom-button"
      >
        Max
      </div>
    </slot>
  </div>
</template>

<script>
export default {
  props: {
    value: {
      type: Number,
      required: true,
    },
    gradient: {
      type: Function,
      requried: true
    },
    min: {
      type: Number,
      required: true,
    },
    max: {
      type: Number,
      required: true,
    },
    lineHeight: {
      type: Number,
      default: 8,
    },
    lineColor: {
      type: [Object, String],
      default: 'white'
    },
    withButtons: {
      type: Boolean,
      default: false,
    }
  },
  data() {
    return {
      ctx: null,
    }
  },
  mounted() {
    this.init()
    this.addListeners()
  },
  methods: {
    init() {
      this.$refs.picker.width = 0
      this.$refs.picker.height = 0

      this.ctx = this.$refs.picker.getContext('2d')
      this.ctx.canvas.width = this.ctx.canvas.clientWidth
      this.ctx.canvas.height = this.ctx.canvas.clientHeight

      this.draw()
    },
    draw() {
      let gradient = this.ctx.createLinearGradient(0, 0, 0, this.ctx.canvas.height)
      this.$props.gradient(gradient)
      this.ctx.fillStyle = gradient
      this.ctx.fillRect(0, 0, this.ctx.canvas.width, this.ctx.canvas.height)

      let position = this.$props.value / this.$props.max * this.ctx.canvas.height - this.$props.lineHeight / 2
      let borderWidth = 2
      
      this.ctx.fillStyle = 'black'
      this.ctx.fillRect(0, position, this.ctx.canvas.width, this.$props.lineHeight)

      this.ctx.fillStyle = 'white'
      this.ctx.fillRect(0, position + borderWidth, this.ctx.canvas.width, this.$props.lineHeight - 2*borderWidth)
    },
    addListeners() {
      let bounding = this.ctx.canvas.getBoundingClientRect()

      let handleSelect = (clientY) => {
        let y = clientY - bounding.top
        let percent = Math.min(Math.max(y / bounding.height, 0), 1)

        this.$emit('update:value', percent * (this.$props.max - this.$props.min))
      }

      this.ctx.canvas.addEventListener('touchmove', event => {
        event.preventDefault()
        handleSelect(event.changedTouches[0].clientY)
      })
      this.ctx.canvas.addEventListener('touchend', event => {
        handleSelect(event.changedTouches[0].clientY)
        this.$emit('end')
      })
      this.ctx.canvas.addEventListener('click', ({ clientY }) => handleSelect(clientY))
      this.ctx.canvas.addEventListener('mousemove', event => {
        if (event.buttons != 1) return
        handleSelect(event.clientY)
      })
    },
  },
  watch: {
    value(newVal, oldVal) {
      this.draw()
    }
  }
}
</script>

<style>
  .top-button, .bottom-button {
    background: black;
    color: white;
    padding: 10px;
    text-align: center;
    font-size: 18px;
    font-weight: bold;
    text-transform: uppercase;
    cursor: pointer;
  }
</style>

<style scoped>
  .color-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    border: 2px solid white;
    border-radius: 10px;
    overflow: hidden;
  }

  canvas {
    flex: 1;
  }
</style>