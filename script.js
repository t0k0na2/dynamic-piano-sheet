import init, { MidiPlayer } from "./pkg/dynamic_piano_sheet.js";
init().then((wasm) => {
  const canvas = document.getElementById('canvas');
  const ctx = canvas.getContext('2d');

  // MIDIプレイヤーのインスタンスを作成
  const midi_player = MidiPlayer.new();

  // キャンバスのサイズをウィンドウに合わせて調整
  function resizeCanvas() {
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;
  }

  window.addEventListener('load', resizeCanvas);
  window.addEventListener('resize', resizeCanvas);
  resizeCanvas();

  const loop_checkbox = document.getElementById("enable-loop");
  const loop_start_bar_input = document.getElementById("loop-start-bar");
  const loop_end_bar_input = document.getElementById("loop-end-bar");

  const loop_inputs = document.getElementById("loop-inputs");

  // ループ設定のUIロジックとMidiPlayerへの反映
  function update_loop_settings() {
    if (loop_checkbox.checked) {
      const start_bar = Math.max(1, Math.min(loop_start_bar_input.valueAsNumber, midi_player.num_bars()));
      const end_bar = Math.max(start_bar, Math.min(loop_end_bar_input.valueAsNumber, midi_player.num_bars()));
      midi_player.set_loop_bars(start_bar - 1, end_bar - 1);
      loop_inputs.style.display = "flex";
    } else {
      midi_player.set_loop_bars(0, 0);
      loop_inputs.style.display = "none";
    }
  }

  // ループチェックボックスの変更イベント
  loop_checkbox.addEventListener('change', (event) => {
    update_loop_settings();
  });

  loop_start_bar_input.addEventListener('input', (event) => {
    update_loop_settings();
  });

  loop_end_bar_input.addEventListener('input', (event) => {
    update_loop_settings();
  });

  // 表示範囲(ズーム)スライダー
  const display_slider = document.getElementById("display-slider");
  display_slider.addEventListener('input', (event) => {
    midi_player.set_display_range(display_slider.valueAsNumber);
  });

  // 再生位置(小節)スライダー
  const bar_slider = document.getElementById("bar-slider");
  bar_slider.addEventListener('input', (event) => {
    const bar_number = bar_slider.valueAsNumber;
    midi_player.seek_bar(bar_number, true);
  });
  const bar_label = document.getElementById("bar-label");

  // 音量スライダー
  const volume_slider = document.getElementById("volume-slider");
  volume_slider.addEventListener('input', (event) => {
    midi_player.set_volume(volume_slider.valueAsNumber);
  });
  volume_slider.value = midi_player.volume();

  // 再生速度スライダー
  const speed_slider = document.getElementById("speed-slider");
  const speed_label = document.getElementById("speed-label");
  let playbackSpeed = 1.0;
  speed_slider.addEventListener('input', (event) => {
    playbackSpeed = speed_slider.valueAsNumber;
    speed_label.textContent = playbackSpeed.toFixed(1) + "x";
  });

  // MIDIファイルの読み込み処理
  let requested_midi_file = null;

  async function load_midi(file) {
    requested_midi_file = file;
  }

  // MIDIファイル選択イベント
  const midi_open = document.getElementById("midi-open");
  midi_open.addEventListener('change', async (event) => {
    const file = event.target.files[0];
    if (!file)
      return;
    load_midi(file);
  });

  // 再生ボタン
  const play_button = document.getElementById("play-button");
  play_button.addEventListener('click', (event) => {
    if (midi_player.ready())
      midi_player.play();
    else
      alert("MIDIファイルを選択してください");
  });

  // 停止ボタン
  const stop_button = document.getElementById("stop-button");
  stop_button.addEventListener('click', (event) => {
    midi_player.stop();
  });

  // キャンバス操作：ドラッグ開始
  let canvasHold = false;
  canvas.onpointerdown = (e) => {
    if (midi_player) {
      e.preventDefault();
      canvasHold = true;
    }
  }

  // キャンバス操作：ドラッグ中（スクロール）
  canvas.onpointermove = (e) => {
    if (canvasHold) {
      e.preventDefault();
      midi_player.skip(e.movementY * display_slider.valueAsNumber / canvas.height);
    }
  }

  canvas.onpointerup = (e) => {
    if (canvasHold) {
      e.preventDefault();
      canvasHold = false;
    }
  }
  canvas.onpointercancel = (e) => {
    if (canvasHold) {
      e.preventDefault();
      canvasHold = false;
    }
  }

  // キャンバス操作：ホイール（スクロール）
  canvas.onwheel = (e) => {
    e.preventDefault();
    midi_player.skip(e.deltaY * -1 * display_slider.valueAsNumber / canvas.height);
  }

  // ドラッグ＆ドロップでのファイル読み込み
  canvas.ondrop = (ev) => {
    ev.preventDefault();
    const dt = new DataTransfer();
    if (ev.dataTransfer.items) {
      [...ev.dataTransfer.items].forEach(async (item, i) => {
        if (item.kind === "file") {
          const file = item.getAsFile();
          load_midi(file);
          dt.items.add(file);
        }
      });
    }
    else {
      [...ev.dataTransfer.files].forEach(async (file, i) => {
        load_midi(file);
        dt.items.add(file);
      });
    }
    midi_open.files = dt.files;
  }

  canvas.ondragover = (ev) => {
    ev.preventDefault();
  }

  // メインの描画ループ
  let animationId = null;
  let lastTime = 0;
  const renderLoop = async (time) => {
    if (!lastTime)
      lastTime = time;

    // MIDIファイルの読み込み要求があればここで処理、読み込み中に割り込みでrenderloopが回るとmidi_playerが例外を発生することがあるので
    if (requested_midi_file !== null) {
      const file = requested_midi_file;
      requested_midi_file = null;
      try {
        const buffer = await file.arrayBuffer();
        midi_player.load_midi(new Uint8Array(buffer));
        bar_slider.max = midi_player.num_bars() - 1;
        loop_start_bar_input.max = midi_player.num_bars();
        loop_end_bar_input.max = midi_player.num_bars();
      } catch (err) {
        alert("MIDIファイルの読み込みに失敗しました\n" + err);
      }
    }

    const deltaTime = time - lastTime;
    lastTime = time;
    midi_player.tick(deltaTime * playbackSpeed);
    midi_player.render(ctx, 0, 0, canvas.width, canvas.height);

    bar_slider.value = midi_player.current_bar();
    bar_label.textContent = String(bar_slider.valueAsNumber + 1) + "/" + String(midi_player.num_bars());

    animationId = requestAnimationFrame(renderLoop);
  };



  // テスト用にexample.midとtest.sf2を自動読み込み
  if (window.location.hostname === "localhost" || window.location.hostname === "127.0.0.1") {
    // MIDIの読み込み
    fetch('example.mid')
      .then(response => {
        if (response.ok) {
          return response.blob();
        }
      })
      .then(blob => {
        if (blob) {
          const file = new File([blob], "example.mid", { type: "audio/midi" });
          load_midi(file);
        }
      })
      .catch(err => console.log("Auto-load MIDI skipped:", err));

    // SoundFontの読み込み
    fetch('test.sf2')
      .then(response => {
        if (response.ok) {
          return response.arrayBuffer();
        }
      })
      .then(buffer => {
        if (buffer) {
          midi_player.load_soundfont(new Uint8Array(buffer));
          console.log("Auto-loaded test.sf2.");
        }
      })
      .catch(err => console.log("Auto-load SoundFont skipped:", err));
  }

  update_loop_settings();
  renderLoop();
});
