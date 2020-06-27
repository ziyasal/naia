
use std::{
    vec::Vec,
    collections::{VecDeque, HashMap},
    rc::Rc};
use byteorder::{BigEndian, ReadBytesExt};

use crate::{EventType, Event, EventClone, PacketReader, Manifest, EntityType};

pub struct EventManager<T: EventType> {
    queued_outgoing_events: VecDeque<Rc<Box<dyn Event<T>>>>,
    queued_incoming_events: VecDeque<T>,
    sent_events: HashMap<u16, Vec<Rc<Box<dyn Event<T>>>>>
}

impl<T: EventType> EventManager<T> {
    pub fn new() -> Self {
        EventManager {
            queued_outgoing_events: VecDeque::new(),
            queued_incoming_events: VecDeque::new(),
            sent_events: HashMap::new(),
        }
    }

    pub fn notify_packet_delivered(&mut self, packet_index: u16) {
        self.sent_events.remove(&packet_index);
    }

    pub fn notify_packet_dropped(&mut self, packet_index: u16) {
        if let Some(dropped_events_list) = self.sent_events.get(&packet_index) {
            for dropped_event in dropped_events_list.into_iter() {
                self.queued_outgoing_events.push_back(dropped_event.clone());
            }

            self.sent_events.remove(&packet_index);
        }
    }

    pub fn has_outgoing_events(&self) -> bool {
        return self.queued_outgoing_events.len() != 0;
    }

    pub fn pop_outgoing_event(&mut self, packet_index: u16) -> Option<Rc<Box<dyn Event<T>>>> {
        match self.queued_outgoing_events.pop_front() {
            Some(event) => {

                //place in transmission record if this is a gauranteed event
                if Event::is_guaranteed(event.as_ref().as_ref()) {
                    if !self.sent_events.contains_key(&packet_index) {
                        let sent_events_list: Vec<Rc<Box<dyn Event<T>>>> = Vec::new();
                        self.sent_events.insert(packet_index, sent_events_list);
                    }

                    if let Some(sent_events_list) = self.sent_events.get_mut(&packet_index) {
                        sent_events_list.push(event.clone());
                    }
                }

                Some(event)
            }
            None => None
        }
    }

    pub fn unpop_outgoing_event(&mut self, packet_index: u16, event: &Rc<Box<dyn Event<T>>>) {
        let cloned_event = event.clone();

        if Event::is_guaranteed(event.as_ref().as_ref()) {
            if let Some(sent_events_list) = self.sent_events.get_mut(&packet_index) {
                sent_events_list.pop();
                if sent_events_list.len() == 0 {
                    self.sent_events.remove(&packet_index);
                }
            }
        }

        self.queued_outgoing_events.push_front(cloned_event);
    }

    pub fn queue_outgoing_event(&mut self, event: &impl Event<T>) {
        let clone = Rc::new(EventClone::clone_box(event));
        self.queued_outgoing_events.push_back(clone);
    }

    pub fn has_incoming_events(&self) -> bool {
        return self.queued_incoming_events.len() != 0;
    }

    pub fn pop_incoming_event(&mut self) -> Option<T> {
        return self.queued_incoming_events.pop_front();
    }

    pub fn process_data<U: EntityType>(&mut self, reader: &mut PacketReader, manifest: &Manifest<T, U>) {
        let buffer = reader.get_buffer();
        let cursor = reader.get_cursor();

        let event_count = cursor.read_u8().unwrap();
        for _x in 0..event_count {
            let naia_id: u16 = cursor.read_u16::<BigEndian>().unwrap().into();
            let payload_length: u8 = cursor.read_u8().unwrap().into();
            let payload_start_position: usize = cursor.position() as usize;
            let payload_end_position: usize = payload_start_position + (payload_length as usize);

            let event_payload = buffer[payload_start_position..payload_end_position]
                .to_vec()
                .into_boxed_slice();

            match manifest.create_event(naia_id, &event_payload) {
                Some(new_event) => {
                    //new_entity.read(&event_payload);
                    self.queued_incoming_events.push_back(new_event);
                }
                _ => {}
            }

            cursor.set_position(payload_end_position as u64);
        }
    }
}