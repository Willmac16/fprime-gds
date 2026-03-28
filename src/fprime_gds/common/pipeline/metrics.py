"""
metrics.py:

Instrumentation module for monitoring GDS pipeline health. Tracks queue depths,
throughput, and latency at each stage of the data pipeline to identify sources
of lag or data loss.

Stages monitored:
    1. Downlinker outgoing queue (comm process: deframed packets waiting to be sent to GDS)
    2. Distributor buffer (GDS process: raw bytes waiting to be parsed into messages)
    3. Decoder throughput (events and channels decoded per interval)
    4. History sizes (items accumulated in RAM histories)
"""

import logging
import threading
import time

LOGGER = logging.getLogger("gds_metrics")


class PipelineMetrics:
    """Collects and reports metrics from the GDS pipeline.

    This class is designed to be attached to the StandardPipeline and periodically
    report on queue depths, throughput counters, and history sizes.
    """

    def __init__(self, interval=2.0):
        """Initialize the metrics collector.

        Args:
            interval: seconds between metric reports
        """
        self.interval = interval
        self._lock = threading.Lock()
        self._stop_event = threading.Event()
        self._thread = None

        # Counters (atomically updated via lock)
        self._events_decoded = 0
        self._channels_decoded = 0
        self._packets_received = 0  # raw packets from distributor
        self._bytes_received = 0

        # References to monitored objects (set during attach)
        self._distributor = None
        self._event_history = None
        self._channel_history = None
        self._command_history = None
        self._downlinker_queue = None  # Queue object from Downlinker

    def attach(self, pipeline):
        """Attach to a StandardPipeline to monitor its components.

        Args:
            pipeline: StandardPipeline instance
        """
        self._distributor = pipeline.distributor
        if pipeline.histories:
            self._event_history = pipeline.histories.events
            self._channel_history = pipeline.histories.channels
            self._command_history = pipeline.histories.commands

        # Wrap decoders to count throughput
        if pipeline.coders.event_decoder:
            self._wrap_decoder(pipeline.coders.event_decoder, "event")
        if pipeline.coders.channel_decoder:
            self._wrap_decoder(pipeline.coders.channel_decoder, "channel")

        # Wrap distributor to count raw packets and bytes
        self._wrap_distributor(pipeline.distributor)

    def attach_downlinker_queue(self, downlinker_queue):
        """Attach a Downlinker's outgoing queue for monitoring.

        Args:
            downlinker_queue: Queue instance from Downlinker.outgoing
        """
        self._downlinker_queue = downlinker_queue

    def _wrap_decoder(self, decoder, decoder_type):
        """Wrap a decoder's send_to_all to count decoded items."""
        original_send_to_all = decoder.send_to_all

        def counted_send_to_all(data, sender=None):
            with self._lock:
                if decoder_type == "event":
                    self._events_decoded += 1
                elif decoder_type == "channel":
                    self._channels_decoded += 1
            return original_send_to_all(data, sender)

        decoder.send_to_all = counted_send_to_all

    def _wrap_distributor(self, distributor):
        """Wrap the distributor's on_recv to count incoming packets and bytes."""
        original_on_recv = distributor.on_recv

        def counted_on_recv(data):
            with self._lock:
                self._packets_received += 1
                self._bytes_received += len(data)
            return original_on_recv(data)

        distributor.on_recv = counted_on_recv

    def start(self):
        """Start the periodic metrics reporting thread."""
        self._thread = threading.Thread(
            target=self._report_loop, name="PipelineMetricsThread", daemon=True
        )
        self._thread.start()

    def stop(self):
        """Stop the metrics reporting thread."""
        self._stop_event.set()
        if self._thread is not None:
            self._thread.join(timeout=self.interval + 1)

    def _report_loop(self):
        """Periodically collect and log metrics."""
        last_events = 0
        last_channels = 0
        last_packets = 0
        last_bytes = 0
        last_time = time.monotonic()

        while not self._stop_event.is_set():
            self._stop_event.wait(self.interval)
            if self._stop_event.is_set():
                break

            now = time.monotonic()
            elapsed = now - last_time
            last_time = now

            with self._lock:
                cur_events = self._events_decoded
                cur_channels = self._channels_decoded
                cur_packets = self._packets_received
                cur_bytes = self._bytes_received

            # Compute rates
            event_rate = (cur_events - last_events) / elapsed if elapsed > 0 else 0
            channel_rate = (cur_channels - last_channels) / elapsed if elapsed > 0 else 0
            packet_rate = (cur_packets - last_packets) / elapsed if elapsed > 0 else 0
            byte_rate = (cur_bytes - last_bytes) / elapsed if elapsed > 0 else 0

            last_events = cur_events
            last_channels = cur_channels
            last_packets = cur_packets
            last_bytes = cur_bytes

            # Collect queue/buffer depths
            distributor_buf_size = 0
            if self._distributor is not None:
                # Access internal buffer size (it's name-mangled)
                buf = getattr(self._distributor, "_Distributor__buf", None)
                if buf is not None:
                    distributor_buf_size = len(buf)

            event_hist_size = self._event_history.size() if self._event_history else 0
            channel_hist_size = self._channel_history.size() if self._channel_history else 0
            cmd_hist_size = self._command_history.size() if self._command_history else 0

            downlinker_q_size = 0
            if self._downlinker_queue is not None:
                downlinker_q_size = self._downlinker_queue.qsize()

            LOGGER.info(
                "METRICS | "
                "rates(evt=%.1f/s ch=%.1f/s pkt=%.1f/s bytes=%.0f/s) | "
                "totals(evt=%d ch=%d pkt=%d) | "
                "backlog(dist_buf=%d bytes, dl_queue=%d frames) | "
                "history(evt=%d ch=%d cmd=%d)",
                event_rate,
                channel_rate,
                packet_rate,
                byte_rate,
                cur_events,
                cur_channels,
                cur_packets,
                distributor_buf_size,
                downlinker_q_size,
                event_hist_size,
                channel_hist_size,
                cmd_hist_size,
            )

    def get_snapshot(self):
        """Return a dict of current metrics for programmatic access.

        Returns:
            dict with current metric values
        """
        with self._lock:
            snapshot = {
                "events_decoded": self._events_decoded,
                "channels_decoded": self._channels_decoded,
                "packets_received": self._packets_received,
                "bytes_received": self._bytes_received,
            }

        if self._distributor is not None:
            buf = getattr(self._distributor, "_Distributor__buf", None)
            snapshot["distributor_buf_bytes"] = len(buf) if buf is not None else 0

        snapshot["event_history_size"] = self._event_history.size() if self._event_history else 0
        snapshot["channel_history_size"] = self._channel_history.size() if self._channel_history else 0
        snapshot["command_history_size"] = self._command_history.size() if self._command_history else 0
        snapshot["downlinker_queue_size"] = self._downlinker_queue.qsize() if self._downlinker_queue else 0

        return snapshot
